use macroquad::prelude::*;
use std::collections::HashMap;

pub struct AnimationSpriteData {
    pub columns: f32,
    pub rows: f32,
    pub true_size: Vec2,
    pub texture: Texture2D,
}

pub struct AnimationData {
    pub start_frame: i32,
    pub end_frame: i32,
    pub loop_frame: Option<i32>,
    pub fps: f32,
}

impl AnimationData {
    pub fn total_time(&self) -> f32 {
        let start = if let Some(loop_frame) = self.loop_frame {
            loop_frame
        } else {
            self.start_frame
        };
        (self.end_frame - start) as f32
    }
}

pub struct AnimationInstance<A>
where
    A: Sized,
{
    pub timer: f32,
    pub sprite_data: AnimationSpriteData,
    pub animations: HashMap<A, AnimationData>,
    pub current_animation: A,
    // auto player this animation after current finishes
    pub into_animation_optional: Option<A>,
    pub scale: Vec2,
}

impl<A> AnimationInstance<A>
where
    A: std::cmp::Eq + std::hash::Hash,
{
    pub fn new(columns: f32, rows: f32, texture: Texture2D, start_animation: A) -> Self {
        let true_size = vec2(texture.width() / columns, texture.height() / rows);
        Self {
            timer: 0f32,
            sprite_data: AnimationSpriteData {
                columns,
                rows,
                true_size,
                texture,
            },
            animations: HashMap::new(),
            current_animation: start_animation,
            into_animation_optional: None,
            scale: vec2(1., 1.),
        }
    }
    pub fn add_animation(
        &mut self,
        start_frame: i32,
        end_frame: i32,
        loop_frame: Option<i32>,
        fps: f32,
        identifier: A,
    ) {
        self.animations.insert(
            identifier,
            AnimationData {
                start_frame,
                end_frame,
                loop_frame,
                fps,
            },
        );
    }

    pub fn update(&mut self, dt: f32) {
        let animation_data = self
            .animations
            .get(&self.current_animation)
            .expect("NO ANIMATION");
        self.timer += dt * animation_data.fps;

        let start_frame = if let Some(loop_frame) = animation_data.loop_frame {
            loop_frame
        } else {
            animation_data.start_frame
        };
        if self.timer > start_frame as f32 + animation_data.total_time() + 1f32 {
            if let Some(loop_frame) = animation_data.loop_frame {
                self.timer = loop_frame as f32;
            } else {
                self.timer = animation_data.start_frame as f32;
            }
            if let Some(into_animation) = self.into_animation_optional.take() {
                self.play_animation(into_animation);
            }
        }
    }

    pub fn play_animation(&mut self, identifier: A) {
        self.current_animation = identifier;
        let animation_data = self
            .animations
            .get(&self.current_animation)
            .expect("NO ANIMATION");
        self.timer = animation_data.start_frame as f32;
    }

    // playe animation then the second one right after
    pub fn play_animation_then(&mut self, identifier: A, after: A) {
        self.current_animation = identifier;
        self.into_animation_optional = Some(after);
        let animation_data = self
            .animations
            .get(&self.current_animation)
            .expect("NO ANIMATION");
        self.timer = animation_data.start_frame as f32;
    }

    pub fn draw(&self, pos: &Vec2, flip_x: bool) {
        let x_index = self.timer as i32 % (self.sprite_data.columns) as i32;
        let y_index = (self.timer as f32 / self.sprite_data.columns).floor();

        draw_texture_ex(
            self.sprite_data.texture,
            pos.x - self.sprite_data.true_size.x * self.scale.x * 0.5f32,
            pos.y - self.sprite_data.true_size.y * self.scale.y * 0.5f32,
            WHITE,
            DrawTextureParams {
                flip_x,
                dest_size: Some(vec2(
                    self.sprite_data.true_size.x * self.scale.x,
                    self.sprite_data.true_size.y * self.scale.y,
                )),
                source: Some(Rect {
                    x: x_index as f32 * self.sprite_data.true_size.x,
                    y: y_index as f32 * self.sprite_data.true_size.y,
                    w: self.sprite_data.true_size.x,
                    h: self.sprite_data.true_size.y,
                }),
                ..Default::default()
            },
        );
    }
}
