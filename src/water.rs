use macroquad::prelude::*;
use miniquad::*;

pub struct Water {
    pub pos: Vec2,
    size: Vec2,
    pub rotation: f32,

    pub direction: Vec2,
    pub speed: f32,
    pub strength: f32,

    offset: Vec2,
    stage: Stage,
}

#[repr(C)]
struct Vertex {
    pos: Vec2,
    uv: Vec2,
    sample_uv: Vec2,
}

struct Stage {
    pipeline: Pipeline,
    bindings: Bindings,
}

#[repr(C)]
pub struct Uniforms {
    strength: f32,
    offset: Vec2,
    pub view: glam::Mat4,
    pub model: glam::Mat4,
}

impl Stage {
    pub fn new(
        ctx: &mut Context,
        tex_water_normal: Texture2D,
        tex_target: Texture2D,
        flip_y: bool,
    ) -> Stage {
        #[rustfmt::skip]
        let vertices = match flip_y {
            true => {
                [
                    Vertex { pos : Vec2::new(-1.0, -1.0), uv: Vec2::new(0., 0.), sample_uv: Vec2::new(-1., -1.) },
                    Vertex { pos : Vec2::new( 1.0, -1.0), uv: Vec2::new(1., 0.), sample_uv: Vec2::new(1., -1.)},
                    Vertex { pos : Vec2::new( 1.0,  1.0), uv: Vec2::new(1., 1.), sample_uv: Vec2::new(1., -3.) },
                    Vertex { pos : Vec2::new(-1.0,  1.0), uv: Vec2::new(0., 1.), sample_uv: Vec2::new(-1., -3.) },
                ]
            },
            false => {
                [
                    Vertex { pos : Vec2::new(-1.0,-1.0 ), uv: Vec2::new(0., 0. ), sample_uv: Vec2::new( -1., -3.) },
                    Vertex { pos : Vec2::new( 1.0,-1.0 ), uv: Vec2::new(1., 0. ), sample_uv: Vec2::new( 1., -3.) },
                    Vertex { pos : Vec2::new( 1.0, 1.0 ), uv: Vec2::new(1., 1. ), sample_uv: Vec2::new( 1., -1.) },
                    Vertex { pos : Vec2::new(-1.0, 1.0 ), uv: Vec2::new(0., 1. ), sample_uv: Vec2::new( -1., -1.) },
                ]
            }
        };

        let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices);

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &indices);

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: vec![
                tex_target.raw_miniquad_texture_handle(),
                tex_water_normal.raw_miniquad_texture_handle(),
            ],
        };

        let shader_meta = ShaderMeta {
            images: vec!["tex_water_normal".to_string(), "texture".to_string()],
            uniforms: UniformBlockLayout {
                uniforms: vec![
                    UniformDesc::new("strength", UniformType::Float1),
                    UniformDesc::new("offset", UniformType::Float2),
                    UniformDesc::new("view", UniformType::Mat4),
                    UniformDesc::new("model", UniformType::Mat4),
                ],
            },
        };

        let shader =
            Shader::new(ctx, WATER_VERTEX_SHADER, WATER_FRAGMENT_SHADER, shader_meta).unwrap();

        let pipeline = Pipeline::new(
            ctx,
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("pos", VertexFormat::Float2),
                VertexAttribute::new("uv", VertexFormat::Float2),
                VertexAttribute::new("sample_uv", VertexFormat::Float2),
            ],
            shader,
        );

        Stage { pipeline, bindings }
    }
}

impl Water {
    pub fn new(
        pos: Vec2,
        size: Vec2,
        tex_water_normal: Texture2D,
        tex_target: Texture2D,
        direction: Vec2,
        speed: f32,
        strength: f32,
        rotation: f32,
    ) -> Self {
        let ctx = unsafe { get_internal_gl().quad_context };
        let stage = Stage::new(ctx, tex_water_normal, tex_target, true);
        Water {
            pos,
            size,
            speed,
            rotation,
            strength,
            direction: direction.normalize(),
            offset: vec2(0., 0.),
            stage,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.offset += self.direction * dt * self.speed;
    }

    pub fn draw(&self, current_camera: &dyn Camera) {
        let mut gl = unsafe { get_internal_gl() };

        // Ensure that macroquad's shapes are not going to be lost
        gl.flush();
        gl.quad_context.apply_pipeline(&self.stage.pipeline);
        let current_pass = current_camera.render_pass();
        gl.quad_context
            .begin_pass(current_pass, miniquad::PassAction::Nothing);
        gl.quad_context.apply_bindings(&self.stage.bindings);

        let view = current_camera.matrix();

        let half_size = self.size * 0.5f32;
        let model = glam::f32::Mat4::from_scale_rotation_translation(
            vec3(half_size.x, half_size.y, 1.0f32),
            glam::Quat::from_rotation_z(self.rotation),
            vec3(self.pos.x + half_size.x, self.pos.y + half_size.y, 0f32),
        );

        gl.quad_context.apply_uniforms(&Uniforms {
            strength: self.strength,
            offset: self.offset,
            view,
            model,
        });
        gl.quad_context.draw(0, 6, 1);

        gl.quad_context.end_render_pass();
    }
}

const WATER_FRAGMENT_SHADER: &str = "#version 140
    in vec2 v_uv;
    uniform float strength;
    uniform sampler2D texture;
    uniform sampler2D tex_water_normal;
    uniform vec2 offset;

    out vec4 color;

    void main() {
        vec4 water_color = texture2D(tex_water_normal, v_uv+offset);
        vec4 base_color_offset = texture2D(texture, v_uv+(water_color.rg*strength));
        color = base_color_offset;
    }
";

const WATER_VERTEX_SHADER: &str = "#version 140
    in vec2 pos;
    in vec2 uv;
    in vec2 sample_uv;
    out vec2 v_uv;

    uniform mat4 model;
    uniform mat4 view;
    uniform vec2 sample_offset;

    vec2 screen_to_uv(vec2 screen) {
        return screen * 0.5 + vec2(0.5, 0.5);
    }

    void main() {
        mat4 modelview = view * model;
        gl_Position = modelview * vec4(pos, 0.0, 1);
        vec4 uv_sample_offset = modelview * vec4(sample_uv.xy, 0, 1);
        v_uv = screen_to_uv(uv_sample_offset.xy);
    }
";
