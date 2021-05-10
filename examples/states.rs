use async_trait::async_trait;
use macroquad::prelude::*;
use macroquad_tantan_toolbox::resources::*;
use macroquad_tantan_toolbox::states::*;
use std::collections::HashMap;

const GAME_SIZE: Vec2 = const_vec2!([1024f32, 604f32]);

pub struct MenuState;
#[async_trait]
impl State<TransitionData, SharedData> for MenuState {
    async fn on_update(
        &mut self,
        _delta_time: f32,
        _payload: &mut StateManagerPayload<SharedData>,
    ) -> Option<StateManagerCommand<TransitionData, SharedData>> {
        if is_key_pressed(KeyCode::Space) {
            return Some(StateManagerCommand::ChangeStateEx(
                Box::new(LoadingState::new(Box::new(GameState))),
                TransitionTime(0.8),
                TransitionData::Spiral,
            ));
        }
        None
    }
    fn on_draw(&mut self, _payload: StateManagerPayload<SharedData>) {
        clear_background(WHITE);
        draw_text(
            "MENU STATE",
            GAME_SIZE.x * 0.5f32 - 70f32,
            GAME_SIZE.y * 0.5f32,
            40f32,
            BLACK,
        );
    }
}

pub struct GameState;
#[async_trait]
impl State<TransitionData, SharedData> for GameState {
    async fn on_update(
        &mut self,
        _delta_time: f32,
        _shared_data: &mut StateManagerPayload<SharedData>,
    ) -> Option<StateManagerCommand<TransitionData, SharedData>> {
        if is_key_pressed(KeyCode::Space) {
            return Some(StateManagerCommand::ChangeStateEx(
                Box::new(LoadingState::new(Box::new(MenuState))),
                TransitionTime(0.3),
                TransitionData::Split,
            ));
        }
        None
    }
    fn on_draw(&mut self, _payload: StateManagerPayload<SharedData>) {
        clear_background(YELLOW);
        draw_text(
            "GAME STATE",
            GAME_SIZE.x * 0.5f32 - 70f32,
            GAME_SIZE.y * 0.5f32,
            40f32,
            BLACK,
        );
    }
}

pub struct LoadingState {
    // optional because we need to consume the internal value when calling changeState
    into_state: Option<Box<dyn State<TransitionData, SharedData>>>,
}

impl LoadingState {
    pub fn new(into_state: Box<dyn State<TransitionData, SharedData>>) -> Self {
        Self {
            into_state: Some(into_state),
        }
    }
}

#[derive(Hash, Eq, Clone, Debug, Copy, PartialEq)]
pub enum TextureIdentifier {
    Player,
    Moose,
}

pub struct TextureResources {
    _player: Texture2D,
    _moose: Texture2D,
}

impl Resources<TextureIdentifier, Texture2D, DefaultFactory> for TextureResources {
    fn build(
        builder: &mut ResourceBuilder<TextureIdentifier, Self, Texture2D, DefaultFactory>,
    ) -> Self {
        Self {
            _player: builder.get_or_panic(TextureIdentifier::Player),
            _moose: builder.get_or_panic(TextureIdentifier::Moose),
        }
    }
}

// BootState will load textures asyncronously whilst drawing the procentage process
// when every texture resource is loaded, transition to into_state
pub struct BootState {
    into_state: Option<Box<dyn State<TransitionData, SharedData>>>,
    texture_resource_builder:
        ResourceBuilder<TextureIdentifier, TextureResources, Texture2D, DefaultFactory>,
}

impl BootState {
    pub fn new(into_state: Box<dyn State<TransitionData, SharedData>>) -> Self {
        Self {
            into_state: Some(into_state),
            texture_resource_builder: ResourceBuilder::<
                TextureIdentifier,
                TextureResources,
                Texture2D,
                DefaultFactory,
            >::new(
                [
                    (TextureIdentifier::Player, "examples/resources/moose.png"),
                    (TextureIdentifier::Moose, "examples/resources/moose.png"),
                ]
                .into(),
            ),
        }
    }
}

#[async_trait]
impl State<TransitionData, SharedData> for BootState {
    fn on_enter(&mut self, _payload: StateManagerPayload<SharedData>) {}

    async fn on_update(
        &mut self,
        _delta_time: f32,
        payload: &mut StateManagerPayload<SharedData>,
    ) -> Option<StateManagerCommand<TransitionData, SharedData>> {
        // load all textures
        let is_done_loading = self.texture_resource_builder.load_next().await;
        if !is_done_loading {
            return None;
        }
        payload.shared_data.texture_resources_optional =
            Some(self.texture_resource_builder.build());
        // unwrap should be safe
        let into_state = self.into_state.take().unwrap();
        return Some(StateManagerCommand::ChangeStateEx(
            into_state,
            TransitionTime(0.3),
            TransitionData::Slide,
        ));
    }
    fn on_draw(&mut self, _shared_data: StateManagerPayload<SharedData>) {
        clear_background(BLACK);
        draw_text(
            format!(
                "BOOTING UP... {:.0}%",
                self.texture_resource_builder.progress() * 100f32
            )
            .as_str(),
            GAME_SIZE.x * 0.5f32 - 140f32,
            GAME_SIZE.y * 0.5f32,
            40f32,
            WHITE,
        );
    }
}

// loading state basically is just a scene inbetween the actual scene we want to load
#[async_trait]
impl State<TransitionData, SharedData> for LoadingState {
    async fn on_update(
        &mut self,
        _delta_time: f32,
        _payload: &mut StateManagerPayload<SharedData>,
    ) -> Option<StateManagerCommand<TransitionData, SharedData>> {
        // unwrap should be safe
        let into_state = self.into_state.take().unwrap();
        return Some(StateManagerCommand::ChangeStateEx(
            into_state,
            TransitionTime(0.3),
            TransitionData::Slide,
        ));
    }
    fn on_draw(&mut self, _shared_data: StateManagerPayload<SharedData>) {
        clear_background(BLACK);
        draw_text(
            format!(
                //"loading level... {:.0}%",
                "fancy transition huh?"
                //(self.download_progress as f32 / self.total_download as f32) * 100f32
            )
            .as_str(),
            GAME_SIZE.x * 0.5f32 - 140f32,
            GAME_SIZE.y * 0.5f32,
            40f32,
            WHITE,
        );
    }
}

pub struct SharedData {
    texture_resources_optional: Option<TextureResources>,
}

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub enum TransitionData {
    Slide,
    Split,
    Spiral,
}

impl Default for TransitionData {
    fn default() -> Self {
        TransitionData::Slide
    }
}

#[macroquad::main("states")]
async fn main() {
    let render_target_game = render_target(GAME_SIZE.x as u32, GAME_SIZE.y as u32);
    render_target_game.texture.set_filter(FilterMode::Nearest);

    let camera2d = Camera2D {
        zoom: vec2(1. / GAME_SIZE.x * 2., 1. / GAME_SIZE.y * 2.),
        target: vec2(
            (GAME_SIZE.x * 0.5f32).floor(),
            (GAME_SIZE.y * 0.5f32).floor(),
        ),
        render_target: Some(render_target_game),
        ..Default::default()
    };

    let loadingstate_menu = Box::new(LoadingState::new(Box::new(MenuState)));
    let boot_state = Box::new(BootState::new(loadingstate_menu));
    let size = RenderTargetSize {
        width: GAME_SIZE.x as u32,
        height: GAME_SIZE.y as u32,
    };
    let transition_tex_split: Texture2D = load_texture("examples/resources/transition_split.png")
        .await
        .unwrap();
    let transition_tex_slide: Texture2D = load_texture("examples/resources/transition_slide.png")
        .await
        .unwrap();
    let transition_tex_spiral: Texture2D = load_texture("examples/resources/transition_spiral.png")
        .await
        .unwrap();
    let shared_data = SharedData {
        texture_resources_optional: None,
    };

    let mut transition_texture_map = HashMap::new();
    transition_texture_map.insert(TransitionData::Split, transition_tex_split);
    transition_texture_map.insert(TransitionData::Slide, transition_tex_slide);
    transition_texture_map.insert(TransitionData::Spiral, transition_tex_spiral);
    let mut state_manager: StateManager<TransitionData, SharedData> = StateManager::new(
        boot_state,
        size,
        camera2d,
        shared_data,
        transition_texture_map,
    );

    loop {
        state_manager.update(get_frame_time()).await;
        state_manager.draw();
        next_frame().await
    }
}
