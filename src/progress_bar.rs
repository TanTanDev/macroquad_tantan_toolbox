use macroquad::prelude::*;

pub struct DrawParam {}

impl Default for DrawParam {
    fn default() -> Self {
        DrawParam {}
    }
}

pub struct ProgressBar {
    pub material: Material,
    pub texture: Texture2D,
    pub texture_border: Texture2D,
    pub scroll_speed: f32,
    pub inner_offset: Vec2,
}

impl ProgressBar {
    pub fn draw(&mut self, pos: Vec2, progress: f32) {
        self.draw_ex(pos, progress, DrawParam::default());
    }

    pub fn draw_ex(&mut self, pos: Vec2, progress: f32, _draw_param: DrawParam) {
        self.material.set_uniform("progress", progress);
        self.material
            .set_uniform("uv_offset", get_time() as f32 * self.scroll_speed);
        draw_texture(self.texture_border, pos.x, pos.y, WHITE);
        gl_use_material(self.material);
        draw_texture_ex(
            self.texture,
            pos.x + self.inner_offset.x,
            pos.y + self.inner_offset.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(233., 41.)),
                ..Default::default()
            },
        );
        gl_use_default_material();
    }

    pub fn new(texture: Texture2D, texture_border: Texture2D, inner_offset: Vec2) -> Self {
        let fragment_shader = FRAGMENT_SHADER.to_string();
        let vertex_shader = VERTEX_SHADER.to_string();

        let pipeline_params = PipelineParams {
            depth_write: true,
            depth_test: Comparison::LessOrEqual,
            ..Default::default()
        };

        let material = load_material(
            &vertex_shader,
            &fragment_shader,
            MaterialParams {
                //textures: vec!["tex_bar".to_string()],
                textures: vec![],
                uniforms: vec![
                    ("progress".to_string(), UniformType::Float1),
                    ("uv_offset".to_string(), UniformType::Float1),
                ],
                pipeline_params,
            },
        )
        .unwrap();

        ProgressBar {
            material,
            texture,
            texture_border,
            scroll_speed: 0.5f32,
            inner_offset,
        }
    }
}

const FRAGMENT_SHADER: &str = "#version 100
    precision lowp float;
    varying vec2 uv;

    uniform float progress;
    uniform float uv_offset;
    uniform sampler2D Texture;

    varying vec4 color;

    void main() {
        if (uv.x > progress) {
            discard;
        }
        vec4 base_color = texture2D(Texture, uv+vec2(uv_offset, 0));

        gl_FragColor = base_color; 
    }
";

const VERTEX_SHADER: &str = "#version 100
    attribute vec3 position;
    attribute vec2 texcoord;
    attribute vec4 color0;

    varying lowp vec2 uv;
    varying lowp vec4 color;
    uniform mat4 Model;
    uniform mat4 Projection;

    void main() {
        gl_Position = Projection * Model * vec4(position, 1);
        color = color0 / 255.0;
        uv = texcoord;
    }
";
