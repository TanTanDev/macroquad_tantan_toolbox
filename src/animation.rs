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

    pub fn draw(&self, pos: &Vec2) {
        let x_index = self.timer as i32 % (self.sprite_data.columns) as i32;
        let y_index = (self.timer as f32 / self.sprite_data.columns).floor();

        draw_texture_ex(
            self.sprite_data.texture,
            pos.x,
            pos.y,
            WHITE,
            DrawTextureParams {
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
