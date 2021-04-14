use async_trait::async_trait;
use macroquad::prelude::*;
use macroquad_tantan_toolbox::states::*;

const GAME_SIZE: Vec2 = Vec2 {
    x: 1024f32,
    y: 604f32,
};

// the menu state prints when something happends AND
// every update will promt a change to GameState
pub struct MenuState;
#[async_trait]
impl State<TransitionData, SharedData> for MenuState {
    async fn on_update(
        &mut self,
        _delta_time: f32,
        _shared_data: &mut SharedData,
    ) -> Option<StateManagerCommand<TransitionData, SharedData>> {
        if is_key_pressed(KeyCode::Space) {
            return Some(StateManagerCommand::ChangeStateEx(
                Box::new(LoadingState::new(Box::new(GameState))),
                TransitionTime(0.3),
                TransitionData::Spiral,
            ));
        }
        None
    }
    fn on_draw(&mut self, _shared_data: &mut SharedData) {
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

// GameState prints when something happends
pub struct GameState;
#[async_trait]
impl State<TransitionData, SharedData> for GameState {
    async fn on_update(
        &mut self,
        _delta_time: f32,
        _shared_data: &mut SharedData,
    ) -> Option<StateManagerCommand<TransitionData, SharedData>> {
        if is_key_pressed(KeyCode::Space) {
            return Some(StateManagerCommand::ChangeStateEx(
                Box::new(LoadingState::new(Box::new(MenuState))),
                TransitionTime(0.3),
                TransitionData::Push,
            ));
        }
        None
    }
    fn on_draw(&mut self, _shared_data: &mut SharedData) {
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
    download_progress: i32,
    total_download: i32,
}

impl LoadingState {
    pub fn new(into_state: Box<dyn State<TransitionData, SharedData>>) -> Self {
        Self {
            into_state: Some(into_state),
            download_progress: 0,
            total_download: 0,
        }
    }
}

#[async_trait]
impl State<TransitionData, SharedData> for LoadingState {
    fn on_enter(&mut self, _shared_data: &mut SharedData) {
        self.download_progress = 0;
        self.total_download = 100;
    }
    async fn on_update(
        &mut self,
        _delta_time: f32,
        shared_data: &mut SharedData,
    ) -> Option<StateManagerCommand<TransitionData, SharedData>> {
        // unload all textures
        for tex in shared_data.game_resources.textures.iter() {
            delete_texture(*tex);
        }
        // load textures
        if self.download_progress < self.total_download {
            self.download_progress += 1;
            return None;
        }
        shared_data.game_resources.textures.clear();

        // unwrap should be safe
        let into_state = self.into_state.take().unwrap();
        return Some(StateManagerCommand::ChangeStateEx(
            into_state,
            TransitionTime(0.3),
            TransitionData::Slide,
        ));
    }
    fn on_draw(&mut self, _shared_data: &mut SharedData) {
        clear_background(BLACK);
        draw_text(
            format!(
                "loading level... {:.0}%",
                (self.download_progress as f32 / self.total_download as f32) * 100f32
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
    game_resources: GameResources,
}

pub struct GameResources {
    pub textures: Vec<Texture2D>,
}

pub enum TransitionData {
    Slide,
    Push,
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
    set_texture_filter(render_target_game.texture, FilterMode::Nearest);

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
    let size = RenderTargetSize {
        width: GAME_SIZE.x as u32,
        height: GAME_SIZE.y as u32,
    };
    let transition_tex: Texture2D = load_texture("examples/resources/transition.png").await;
    let shared_data = SharedData {
        game_resources: GameResources {
            textures: Vec::new(),
        },
    };
    let mut state_manager: StateManager<TransitionData, SharedData> = StateManager::new(
        loadingstate_menu,
        size,
        camera2d,
        transition_tex,
        shared_data,
    );

    loop {
        state_manager.update(get_frame_time()).await;
        state_manager.draw();
        next_frame().await
    }
}
