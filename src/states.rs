use crate::transition;
use crate::transition::*;
use async_trait::async_trait;
use macroquad::prelude::*;

// T: user defined transition data
// S: shared data
#[async_trait]
pub trait State<T, S>
where
    S: Send + Sync,
    Self: Send,
{
    fn on_enter(&mut self, _shared_data: &mut S) {}
    fn on_exit(&mut self, _shared_data: &mut S) {}
    async fn on_update(
        &mut self,
        _delta_time: f32,
        _shared_data: &mut S,
    ) -> Option<StateManagerCommand<T, S>>;
    fn on_draw(&mut self, _shared_data: &mut S) {}
}

pub struct TransitioningData<T, S>
where
    S: Send + Sized,
    Self: Sized + Send,
{
    pub time_left: f32,
    start_time: f32,
    into_state: Box<dyn State<T, S>>,
}

impl<T, S> TransitioningData<T, S>
where
    S: Send + Sized,
    Self: Sized + Send,
{
    pub fn progress(&self) -> f32 {
        self.time_left / self.start_time
    }
}

pub enum TransitionState<T, S>
where
    S: Send + Sized,
    Self: Sized + Send,
{
    None,
    Transitioning(TransitioningData<T, S>),
}

impl<T, S> Default for TransitionState<T, S>
where
    S: Send + Sized,
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

// this statemanager handles transitioning animation built in
// the rendering part is heavily tailored around macroquad
// T: transition data
// S: shared data owned by statemanager
pub struct StateManager<T, S>
where
    S: Send + Sized,
    Self: Sized + Send,
{
    current_state: Box<dyn State<T, S>>,
    transition_state: TransitionState<T, S>,
    _transition_data: T,
    shared_data: S,
    transition: Transition,
    current_rendertarget: RenderTarget,
    into_rendertarget: RenderTarget,
    camera: Camera2D,
}

// T: transition data
impl<T, S> StateManager<T, S>
where
    T: Default,
    S: Send + Sized + Sync,
    Self: Sized + Send,
{
    pub fn new(
        initial_state: Box<dyn State<T, S>>,
        rendertarget_size: RenderTargetSize,
        camera: Camera2D,
        transition_tex: Texture2D,
        shared_data: S,
    ) -> Self {
        let mut state_manager = Self {
            current_state: initial_state,
            transition_state: TransitionState::None,
            _transition_data: T::default(),
            transition: Transition::new(transition_tex, 0.3f32),
            current_rendertarget: render_target(rendertarget_size.width, rendertarget_size.height),
            into_rendertarget: render_target(rendertarget_size.width, rendertarget_size.height),
            shared_data,
            camera,
        };
        state_manager
            .current_state
            .on_enter(&mut state_manager.shared_data);
        state_manager
    }

    // change state instantly without transition
    pub fn change_state(&mut self, state: Box<dyn State<T, S>>) {
        self.current_state.on_exit(&mut self.shared_data);
        self.current_state = state;
        self.current_state.on_enter(&mut self.shared_data);
    }

    // change state with transition
    pub fn change_state_ex(
        &mut self,
        mut state: Box<dyn State<T, S>>,
        time: TransitionTime,
        _transition_data: T,
    ) {
        // called right as we start transitioning...
        state.on_enter(&mut self.shared_data);
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
                    self.current_state.on_exit(&mut self.shared_data);
                    self.current_state = transitioning_data.into_state;
                }
            }
            return;
        }
        let command_optional = self
            .current_state
            .on_update(delta_time, &mut self.shared_data)
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
    }

    fn change_rendertarget(mut camera: Camera2D, target: RenderTarget) {
        camera.render_target = Some(target);
        set_camera(camera);
    }

    // call the current states, draw funciton
    pub fn draw(&mut self) {
        Self::change_rendertarget(self.camera, self.current_rendertarget);
        self.current_state.on_draw(&mut self.shared_data);
        if let TransitionState::Transitioning(transitioning_data) = &mut self.transition_state {
            // draw into state
            Self::change_rendertarget(self.camera, self.into_rendertarget);
            transitioning_data.into_state.on_draw(&mut self.shared_data);

            // combine and draw transition
            set_default_camera();
            clear_background(WHITE);
            self.transition.draw_ex(
                self.current_rendertarget.texture,
                self.into_rendertarget.texture,
                transitioning_data.progress(),
                transition::DrawParam { flip_y: true },
            );
        } else {
            // DRAW CURRENT STATE ONLY
            set_default_camera();
            clear_background(WHITE);
            draw_texture_ex(
                self.current_rendertarget.texture,
                0.,
                0.,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(screen_width(), screen_height())),
                    ..Default::default()
                },
            );
        }
    }
}