use macroquad::prelude::*;
use macroquad_tantan_toolbox::water::*;

const GAME_SIZE: Vec2 = Vec2 {
    x: 1344f32,
    y: 768f32,
};

#[macroquad::main("water")]
async fn main() {
    let render_target_game = render_target(GAME_SIZE.x as u32, GAME_SIZE.y as u32);
    set_texture_filter(render_target_game.texture, FilterMode::Nearest);

    let camera2d = Camera2D {
        zoom: vec2(1. / GAME_SIZE.x * 2., 1. / GAME_SIZE.y * 2.),
        target: vec2(
            (GAME_SIZE.x * 0.5f32).floor() + (get_time().sin() as f32 * 3.0f32),
            (GAME_SIZE.y * 0.5f32).floor(),
        ),
        render_target: Some(render_target_game),
        ..Default::default()
    };

    let tex_water_raw = load_image("examples/resources/water_normal.png").await;
    let tex_water_normal = {
        // All of this is so we can sample the texture repeatedly
        use miniquad;
        use miniquad::{FilterMode, TextureFormat, TextureParams, TextureWrap};
        let ctx = unsafe { get_internal_gl().quad_context };
        let texture_miniquad = miniquad::graphics::Texture::from_data_and_format(
            ctx,
            &tex_water_raw.bytes,
            TextureParams {
                format: TextureFormat::RGBA8,
                wrap: TextureWrap::Repeat,
                filter: FilterMode::Linear,
                width: tex_water_raw.width as u32,
                height: tex_water_raw.height as u32,
            },
        );
        Texture2D::from_miniquad_texture(texture_miniquad)
    };

    let water_size = vec2(GAME_SIZE.x, GAME_SIZE.y * 0.5f32 + 10f32);
    let water_pos = vec2(water_size.x * 0.0f32, GAME_SIZE.y * 0.5f32 + 100f32);
    let water_dir = vec2(1.0f32, 0.0f32);
    let water_speed = 0.05f32;
    let water_strength = 0.02f32;
    let mut water = Water::new(
        water_pos,
        water_size,
        render_target_game.texture,
        tex_water_normal,
        water_dir,
        water_speed,
        water_strength,
        0f32,
    );

    let beautiful = load_texture("examples/resources/dig_escape_snap.png").await;
    set_texture_filter(beautiful, FilterMode::Nearest);
    loop {
        // draw first view to render texture
        set_camera(camera2d);
        clear_background(BLUE);
        draw_texture_ex(
            beautiful,
            0f32,
            -200f32,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(GAME_SIZE.x, GAME_SIZE.y)),
                ..Default::default()
            },
        );

        water.update(get_frame_time());
        water.draw(&camera2d);

        set_default_camera();
        // we wont see yellow because transition is drawn over, but we need to clear anyway
        clear_background(WHITE);

        draw_texture_ex(
            render_target_game.texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );
        draw_ui(&mut water);

        next_frame().await
    }
}

use macroquad::ui::{
    hash, root_ui,
    widgets::{self},
};

fn draw_ui(water: &mut Water) {
    widgets::Window::new(hash!(), vec2(400., 200.), vec2(320., 400.))
        .label("Settings")
        .ui(&mut *root_ui(), |ui| {
            ui.label(Vec2::new(10., 10.), "HAAH");
            ui.slider(hash!(), "strength", 0f32..0.4f32, &mut water.strength);
            ui.slider(hash!(), "speed", 0f32..1f32, &mut water.speed);
        });
}
