use macroquad::prelude::*;
use macroquad_tantan_toolbox::transition;
use macroquad_tantan_toolbox::transition::Transition;

const GAME_SIZE: Vec2 = Vec2 {
    x: 512f32,
    y: 512f32,
};

#[macroquad::main("transition simple")]
async fn main() {
    let render_target_view1 = render_target(GAME_SIZE.x as u32, GAME_SIZE.y as u32);
    set_texture_filter(render_target_view1.texture, FilterMode::Nearest);

    let render_target_view2 = render_target(GAME_SIZE.x as u32, GAME_SIZE.y as u32);
    set_texture_filter(render_target_view2.texture, FilterMode::Nearest);

    let transition_tex: Texture2D = load_texture("examples/resources/transition.png").await;

    let mut camera2d = Camera2D {
        zoom: vec2(0.01, 0.01),
        ..Default::default()
    };

    let fade = 0.1f32;
    let mut transition = Transition::new(transition_tex, fade);
    loop {
        // draw first view to render texture
        camera2d.render_target = Some(render_target_view1);
        set_camera(camera2d);
        clear_background(GREEN);
        draw_circle(0f32, 0f32, 10.0f32, BLACK);
        draw_text("VIEW 1", -50f32, 0f32, 40f32, BLACK);

        // draw second screen to render texture
        camera2d.render_target = Some(render_target_view2);
        set_camera(camera2d);
        clear_background(BLUE);
        draw_text("VIEW 2", -50f32, 0f32, 40f32, WHITE);

        // draw the transition
        set_default_camera();
        // we wont see yellow because transition is drawn over, but we need to clear anyway
        clear_background(YELLOW);
        let progress = (get_time() as f32 * 4.0f32).sin() * 0.5f32 + 0.5f32;

        // flip_y because rendertexture are flipped...
        transition.draw_ex(
            render_target_view1.texture,
            render_target_view2.texture,
            progress,
            transition::DrawParam { flip_y: true },
        );

        next_frame().await
    }
}
