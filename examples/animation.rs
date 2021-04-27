use macroquad::prelude::*;
use macroquad_tantan_toolbox::animation::*;

#[derive(std::hash::Hash, Eq, PartialEq)]
enum MooseAnimationIdentifier {
    Run,
    Sleep,
}

const GAME_SIZE: Vec2 = Vec2 { x: 64f32, y: 64f32 };

#[macroquad::main("animation")]
async fn main() {
    let texture: Texture2D = load_texture("examples/resources/moose.png").await;
    set_texture_filter(texture, FilterMode::Nearest);

    let game_render_target = render_target(GAME_SIZE.x as u32, GAME_SIZE.y as u32);
    set_texture_filter(game_render_target.texture, FilterMode::Nearest);

    let mut animation = AnimationInstance::<MooseAnimationIdentifier>::new(
        10f32,
        1f32,
        texture,
        MooseAnimationIdentifier::Run,
    );
    animation.add_animation(0, 3, None, 15f32, MooseAnimationIdentifier::Run);
    animation.add_animation(4, 9, Some(7), 13f32, MooseAnimationIdentifier::Sleep);

    loop {
        set_camera(Camera2D {
            zoom: vec2(1. / GAME_SIZE.x * 2., 1. / GAME_SIZE.y * 2.),
            target: vec2(0.0, 0.0),
            render_target: Some(game_render_target),
            ..Default::default()
        });
        clear_background(BLUE);

        // change animation
        if is_key_pressed(KeyCode::Space) {
            let next_state = match animation.current_animation {
                MooseAnimationIdentifier::Run => MooseAnimationIdentifier::Sleep,
                MooseAnimationIdentifier::Sleep => MooseAnimationIdentifier::Run,
            };
            animation.play_animation(next_state);
        }

        animation.update(get_frame_time());
        animation.draw(&vec2(0f32, 0f32), false);
        //animation.draw(&vec2(-GAME_SIZE.x * 0.5f32, 0f32), false);

        set_default_camera();
        clear_background(BLUE);
        // draw game
        draw_texture_ex(
            game_render_target.texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );
        draw_text(
            "tap space to change animation",
            screen_width() * 0.5f32 - 100f32,
            40f32,
            30f32,
            BLACK,
        );

        next_frame().await
    }
}
