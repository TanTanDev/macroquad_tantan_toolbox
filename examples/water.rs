use macroquad::prelude::*;
use macroquad_tantan_toolbox::water::*;
use macroquad_tantan_toolbox::water;

const GAME_SIZE: Vec2 = Vec2 {
    x: 512f32,
    y: 512f32,
};

#[macroquad::main("water")]
async fn main() {
    let render_target_game = render_target(GAME_SIZE.x as u32, GAME_SIZE.y as u32);
    set_texture_filter(render_target_game.texture, FilterMode::Nearest);

    let mut camera2d = Camera2D {
        //zoom: vec2(0.01, 0.01),
        zoom: vec2(1. / GAME_SIZE.x * 2., 1. / GAME_SIZE.y * 2.),
        target: vec2(
            (GAME_SIZE.x * 0.5f32).floor(),
            (GAME_SIZE.y * 0.5f32).floor(),
        ),
        render_target: Some(render_target_game),
        ..Default::default()
    };

    let tex_water_raw = load_image("examples/resources/water_normal.png").await;
    let tex_water_normal = {
        // All of this is so we can sample the texture repeatedly
        use miniquad;
        use miniquad::{TextureFormat, FilterMode, TextureParams, TextureWrap};
        let ctx = unsafe{get_internal_gl().quad_context};
        let texture_miniquad = miniquad::graphics::Texture::from_data_and_format(ctx, &tex_water_raw.bytes, TextureParams{
            format: TextureFormat::RGBA8,
            wrap: TextureWrap::Repeat,
            filter: FilterMode::Linear,
            width: tex_water_raw.width as u32,
            height: tex_water_raw.height as u32,
        });
        Texture2D::from_miniquad_texture(texture_miniquad)
    };

    let water_size = vec2(GAME_SIZE.x*0.5f32, GAME_SIZE.y*0.5f32);
    let water_pos = vec2(water_size.x*0.5f32, GAME_SIZE.y*0.5f32);
    let water_dir = vec2(1.0f32, 0.0f32);
    let water_speed = 0.05f32;
    let water_strength = 0.05f32;
    let mut water = Water::new(water_pos, water_size, tex_water_normal, water_dir, water_speed, water_strength);
    loop {
        // draw first view to render texture
        set_camera(camera2d);
        clear_background(BLUE);

        // draw random stuff
        for i in 0..700 {
            rand::srand(i);
            draw_rectangle(rand::gen_range(0f32, GAME_SIZE.x), rand::gen_range(0f32, GAME_SIZE.y), 10f32, 10f32, Color::new(i as f32/2000f32, 1.0f32, 1.0f32, 1.0f32));
        }

        let ground_height = 100f32; 
        //draw_rectangle(GAME_SIZE.x*0.5f32, GAME_SIZE.y*0.5f32+ground_height*0.5f32, GAME_SIZE.x, ground_height, GREEN);
        draw_rectangle(0f32, GAME_SIZE.y*0.5f32-ground_height, GAME_SIZE.x, ground_height, GREEN);
        water.update(get_frame_time());
        water.draw_ex(render_target_game.texture, water::DrawParam {flip_y: true});

        // draw the transition
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

        next_frame().await
    }
}
