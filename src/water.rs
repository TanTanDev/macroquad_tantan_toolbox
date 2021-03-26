//use macroquad::math::{Vec2};
use macroquad::prelude::*;

pub struct DrawParam {
    pub flip_y: bool,
}

pub struct Water {
    pos: Vec2,
    size: Vec2,

    pub direction: Vec2,
    pub speed: f32,
    pub strength: f32,

    offset: Vec2,
    material: Material,
}

impl Water {
    pub fn new(pos: Vec2, size: Vec2, tex_water_normal: Texture2D, direction: Vec2, speed: f32, strength: f32) -> Self {
        let fragment_shader = WATER_FRAGMENT_SHADER.to_string();
        let vertex_shader = WATER_VERTEX_SHADER.to_string();

        let pipeline_params = PipelineParams {
            depth_write: true,
            depth_test: Comparison::LessOrEqual,
            ..Default::default()
        };

        let material = load_material(
            &vertex_shader,
            &fragment_shader,
            MaterialParams {
                textures: vec!["tex_water_normal".to_string()],
                uniforms: vec![
                    ("strength".to_string(), UniformType::Float1),
                    ("offset".to_string(), UniformType::Float2),
                ],
                pipeline_params,
                ..Default::default()
            },
        )
        .unwrap();

        material.set_texture("tex_water_normal", tex_water_normal);
        material.set_uniform("strength", strength);
        Water {
            pos,
            size,
            material,
            speed,
            strength,
            direction: direction.normalize(),
            offset: vec2(0., 0.),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.offset += self.direction * dt * self.speed;
        self.material.set_uniform("offset", self.offset);
    }

    pub fn draw_ex(&self, base_texture: Texture2D, draw_param: DrawParam) {
        gl_use_material(self.material);
        //draw_rectangle(self.pos.x, self.pos.y, self.size.x, self.size.y, YELLOW);

        let aspect_err = vec2(base_texture.width()/self.size.x, base_texture.height()/self.size.y);
        let pos_fix = vec2(self.pos.x / (1.0/aspect_err.x), self.pos.y * aspect_err.y); 

        draw_texture_ex(
            base_texture,
            0f32,//pos_fix.x,//self.pos.x,
            0f32,//pos_fix.y,//self.pos.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(aspect_err),
                flip_y: draw_param.flip_y,
                ..Default::default()
            },
        );
        gl_use_default_material();
    }
}

const WATER_FRAGMENT_SHADER: &'static str = "#version 140
    in vec2 uv;
    uniform float strength;
    // base texture
    uniform sampler2D Texture;
    uniform sampler2D tex_water_normal;
    uniform vec2 offset;

    out vec4 color;

    void main() {
        vec4 water_color = texture2D(tex_water_normal, uv+offset);
        vec4 base_color_offset = texture2D(Texture, uv+(water_color.rg*strength));
        color = base_color_offset;
    }
";

const WATER_VERTEX_SHADER: &'static str = "#version 140
    in vec3 position;
    in vec2 texcoord;
    out vec2 uv;

    void main() {
        gl_Position = vec4(position, 1);
        uv = texcoord;
    }
";
