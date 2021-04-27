use crate::transition;
use crate::transition::*;
use async_trait::async_trait;
use macroquad::prelude::*;
use std::collections::HashMap;
use std::hash::Hash;

// T: user defined transition data
// S: shared data
#[async_trait]
pub trait State<T, S>
where
    S: Send,
    Self: Send,
{
    fn on_enter(&mut self, _payload: StateManagerPayload<S>) {}
    fn on_exit(&mut self, _payload: StateManagerPayload<S>) {}
    async fn on_update(
        &mut self,
        _delta_time: f32,
        _payload: &mut StateManagerPayload<S>,
    ) -> Option<StateManagerCommand<T, S>>;
    fn on_draw(&mut self, _payload: StateManagerPayload<S>) {}
}

pub struct TransitioningData<T, S>
where
    S: Send + Sized,
    T: Send + Sized,
    Self: Sized + Send,
{
    pub time_left: f32,
    start_time: f32,
    into_state: Box<dyn State<T, S>>,
}

impl<T, S> TransitioningData<T, S>
where
    S: Send + Sized,
    T: Send + Sized,
    Self: Sized + Send,
{
    pub fn progress(&self) -> f32 {
        self.time_left / self.start_time
    }
}

pub enum TransitionState<T, S>
where
    S: Send + Sized,
    T: Send + Sized,
    Self: Sized + Send,
{
    None,
    Transitioning(TransitioningData<T, S>),
}

impl<T, S> Default for TransitionState<T, S>
where
    S: Send + Sized,
    T: Send + Sized,
    Self: Sized + Send,
{
    fn default() -> Self {
        TransitionState::None
    }
}

// Used to take ownership of the TransitioningData
// whilst resetting back to TransitionState::default(), which is TransitionSate::None
impl<T, S> TransitionState<T, S>
where
    S: Send + Sized,
    T: Send + Sized,
    Self: Sized + Send,
{
    pub fn take(&mut self) -> Self {
        std::mem::take(self)
    }
}

pub struct TransitionTime(pub f32);

pub struct RenderTargetSize {
    pub width: u32,
    pub height: u32,
}

pub enum StateManagerCommand<T, S> {
    ChangeState(Box<dyn State<T, S>>),
    ChangeStateEx(Box<dyn State<T, S>>, TransitionTime, T),
}

// passed to states to reference members of statemanager
pub struct StateManagerPayload<'a, S>
where
    S: Send + Sized,
{
    pub shared_data: &'a mut S,
    pub camera: &'a mut Camera2D,
    pub current_rendertarget: &'a mut RenderTarget,
}

// this statemanager handles transitioning animation built in
// the rendering part is heavily tailored around macroquad
// T: transition data
// S: shared data owned by statemanager
pub struct StateManager<T, S>
where
    S: Send + Sized,
    T: Send + Sized,
    Self: Sized + Send,
{
    current_state: Box<dyn State<T, S>>,
    transition_state: TransitionState<T, S>,
    last_transition_data: T,
    pub shared_data: S,
    transition: Transition,
    current_rendertarget: RenderTarget,
    into_rendertarget: RenderTarget,
    camera: Camera2D,
    transition_texture_map: HashMap<T, Texture2D>,

    // callbacks that always run
    pub on_update_optional: Option<fn(&mut Self)>,
    pub on_draw_optional: Option<fn(&mut Self)>,
}

// T: transition data
impl<T, S> StateManager<T, S>
where
    T: Default + Send + Sized + Copy + Eq + PartialEq + Hash,
    S: Send + Sized,
    Self: Sized + Send,
{
    pub fn new(
        initial_state: Box<dyn State<T, S>>,
        rendertarget_size: RenderTargetSize,
        camera: Camera2D,
        shared_data: S,
        transition_texture_map: HashMap<T, Texture2D>,
    ) -> Self {
        let first_transition_tex = transition_texture_map.iter().next().unwrap().1;
        let mut state_manager = Self {
            current_state: initial_state,
            transition_state: TransitionState::None,
            last_transition_data: T::default(),
            transition: Transition::new(*first_transition_tex, 0.3f32),
            current_rendertarget: render_target(rendertarget_size.width, rendertarget_size.height),
            into_rendertarget: render_target(rendertarget_size.width, rendertarget_size.height),
            shared_data,
            camera,
            transition_texture_map,
            on_update_optional: None,
            on_draw_optional: None,
        };
        //state_manager.into_rendertarget.texture.set_filter(FilterMode::Nearest);
        state_manager
            .into_rendertarget
            .texture
            .set_filter(FilterMode::Nearest);
        //state_manager.current_rendertarget.texture.set_filter( FilterMode::Nearest);
        state_manager
            .current_rendertarget
            .texture
            .set_filter(FilterMode::Nearest);
        state_manager.current_state.on_enter(StateManagerPayload {
            shared_data: &mut state_manager.shared_data,
            camera: &mut state_manager.camera,
            current_rendertarget: &mut state_manager.current_rendertarget,
        });
        state_manager
    }

    // change state instantly without transition
    pub fn change_state(&mut self, state: Box<dyn State<T, S>>) {
        self.current_state.on_exit(StateManagerPayload {
            shared_data: &mut self.shared_data,
            camera: &mut self.camera,
            current_rendertarget: &mut self.current_rendertarget,
        });
        self.current_state = state;
        self.current_state.on_enter(StateManagerPayload {
            shared_data: &mut self.shared_data,
            camera: &mut self.camera,
            current_rendertarget: &mut self.current_rendertarget,
        });
    }

    // change state with transition
    pub fn change_state_ex(
        &mut self,
        mut state: Box<dyn State<T, S>>,
        time: TransitionTime,
        transition_data: T,
    ) {
        // update transition texture
        if self.last_transition_data != transition_data {
            let transition_tex = self.transition_texture_map.get(&transition_data).unwrap();
            self.transition.change_transition_tex(*transition_tex);
        }
        self.last_transition_data = transition_data;
        // called right as we start transitioning...
        state.on_enter(StateManagerPayload {
            shared_data: &mut self.shared_data,
            camera: &mut self.camera,
            current_rendertarget: &mut self.current_rendertarget,
        });
        self.transition_state = TransitionState::Transitioning(TransitioningData {
            time_left: time.0,
            start_time: time.0,
            into_state: state,
        });
    }

    // updates the current state, might handle transitioning
    pub async fn update(&mut self, delta_time: f32) {
        if let TransitionState::Transitioning(transitioning_data) = &mut self.transition_state {
            transitioning_data.time_left -= delta_time;
            if transitioning_data.time_left < 0f32 {
                if let TransitionState::Transitioning(transitioning_data) =
                    self.transition_state.take()
                {
                    self.current_state.on_exit(StateManagerPayload {
                        shared_data: &mut self.shared_data,
                        camera: &mut self.camera,
                        current_rendertarget: &mut self.current_rendertarget,
                    });
                    self.current_state = transitioning_data.into_state;
                }
            }
            return;
        }
        let command_optional = self
            .current_state
            .on_update(
                delta_time,
                &mut StateManagerPayload {
                    shared_data: &mut self.shared_data,
                    camera: &mut self.camera,
                    current_rendertarget: &mut self.current_rendertarget,
                },
            )
            .await;
        if let Some(command) = command_optional {
            match command {
                StateManagerCommand::ChangeState(state) => {
                    self.change_state(state);
                }
                StateManagerCommand::ChangeStateEx(state, transition_time, transition_data) => {
                    self.change_state_ex(state, transition_time, transition_data);
                }
            }
        }

        if let Some(update_fn) = self.on_update_optional {
            update_fn(self);
        }
    }

    fn change_rendertarget(mut camera: &mut Camera2D, target: RenderTarget) {
        camera.render_target = Some(target);
        set_camera(camera);
    }

    // call the current states, draw funciton
    pub fn draw(&mut self) {
        Self::change_rendertarget(&mut self.camera, self.current_rendertarget);
        self.current_state.on_draw(StateManagerPayload {
            shared_data: &mut self.shared_data,
            camera: &mut self.camera,
            current_rendertarget: &mut self.current_rendertarget,
        });

        if let TransitionState::Transitioning(transitioning_data) = &mut self.transition_state {
            // draw into state
            Self::change_rendertarget(&mut self.camera, self.into_rendertarget);
            transitioning_data.into_state.on_draw(StateManagerPayload {
                shared_data: &mut self.shared_data,
                camera: &mut self.camera,
                current_rendertarget: &mut self.current_rendertarget,
            });

            // combine and draw transition
            Self::change_rendertarget(&mut self.camera, self.current_rendertarget);
            self.transition.draw_ex(
                self.current_rendertarget.texture,
                self.into_rendertarget.texture,
                transitioning_data.progress(),
                transition::DrawParam { flip_y: false },
            );
        }
        let game_size = vec2(
            self.current_rendertarget.texture.width(),
            self.current_rendertarget.texture.height(),
        );
        let game_diff_w = screen_width() / game_size.x;
        let game_diff_h = screen_height() / game_size.y;
        let aspect_diff = game_diff_w.min(game_diff_h);

        let scaled_game_size_w = game_size.x * aspect_diff;
        let scaled_game_size_h = game_size.y * aspect_diff;

        let width_padding = (screen_width() - scaled_game_size_w) * 0.5f32;
        let height_padding = (screen_height() - scaled_game_size_h) * 0.5f32;
        let dest_size = Some(Vec2::new(scaled_game_size_w, scaled_game_size_h));
        // DRAW CURRENT STATE ONLY
        set_default_camera();
        clear_background(BLACK);
        draw_texture_ex(
            self.current_rendertarget.texture,
            width_padding,
            height_padding,
            WHITE,
            DrawTextureParams {
                dest_size,
                ..Default::default()
            },
        );
    }
}
