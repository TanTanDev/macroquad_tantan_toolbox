use macroquad::prelude::*;

pub struct DrawParam {
    pub flip_y: bool,
}

impl Default for DrawParam {
    fn default() -> Self {
        DrawParam { flip_y: false }
    }
}

pub struct Transition {
    pub material: Material,
    pub fade: f32,
}

impl Transition {
    pub fn draw(&mut self, base_texture: Texture2D, into_texture: Texture2D, progress: f32) {
        self.draw_ex(base_texture, into_texture, progress, DrawParam::default());
    }

    pub fn draw_ex(
        &mut self,
        base_texture: Texture2D,
        into_texture: Texture2D,
        progress: f32,
        draw_param: DrawParam,
    ) {
        self.material.set_uniform("cutoff", progress);
        self.material.set_uniform("fade", self.fade);
        self.material.set_texture("tex_into", into_texture);
        gl_use_material(self.material);
        draw_texture_ex(
            base_texture,
            -1.,
            -1.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(2., 2.)),
                flip_y: draw_param.flip_y,
                ..Default::default()
            },
        );
        gl_use_default_material();
    }

    pub fn change_transition_tex(&mut self, texture: Texture2D) {
        self.material.set_texture("tex_transition", texture);
    }

    pub fn new(transition_tex: Texture2D, fade: f32) -> Self {
        let fragment_shader = DEFAULT_FRAGMENT_SHADER.to_string();
        let vertex_shader = DEFAULT_VERTEX_SHADER.to_string();

        let pipeline_params = PipelineParams {
            depth_write: true,
            depth_test: Comparison::LessOrEqual,
            ..Default::default()
        };

        let material = load_material(
            &vertex_shader,
            &fragment_shader,
            MaterialParams {
                textures: vec!["tex_transition".to_string(), "tex_into".to_string()],
                uniforms: vec![
                    ("cutoff".to_string(), UniformType::Float1),
                    ("fade".to_string(), UniformType::Float1),
                ],
                pipeline_params,
                ..Default::default()
            },
        )
        .unwrap();

        material.set_texture("tex_transition", transition_tex);
        Transition { material, fade }
    }
}

const DEFAULT_FRAGMENT_SHADER: &'static str = "#version 140
    in vec2 uv;

    uniform float cutoff;
    uniform float fade;
    // base texture
    uniform sampler2D Texture;
    uniform sampler2D tex_into;
    uniform sampler2D tex_transition;

    out vec4 color;

    void main() {
        float transition = texture2D(tex_transition, uv).r;
        vec4 base_color = texture2D(Texture, uv);
        vec4 into_color = texture2D(tex_into, uv);

        // remap transition from 0-1 to fade -> 1.0-fade
        transition = transition * (1.0 - fade) + fade;
        float f = smoothstep(cutoff, cutoff + fade, transition);
        color = mix(base_color, into_color, f);
    }
";

const DEFAULT_VERTEX_SHADER: &'static str = "#version 140
    in vec3 position;
    in vec2 texcoord;
    out vec2 uv;

    void main() {
        gl_Position = vec4(position, 1);
        uv = texcoord;
    }
";
