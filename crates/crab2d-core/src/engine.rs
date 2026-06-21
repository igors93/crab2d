use crab2d_assets::AssetRegistry;
use crab2d_platform::InputState;
use crab2d_plugin_api::{EngineContext, Plugin};
use crab2d_scene::{PrefabRegistry, Scene};

use std::path::Path;

use crate::game_flow::GameFlow;
use crate::playtest::PlaytestState;
use crate::runtime_systems::run_scene_systems;
use crate::{
    EngineConfig, EngineTickError, FrameStep, ProjectDocument, ProjectInfo, ProjectIoError,
};

#[derive(Debug)]
pub struct Engine {
    pub config: EngineConfig,
    pub project: ProjectInfo,
    pub assets: AssetRegistry,
    pub active_scene: Scene,
    pub prefabs: PrefabRegistry,
    pub flow: GameFlow,
    pub playtest: PlaytestState,
}

impl Engine {
    pub fn new(config: EngineConfig) -> Self {
        Self {
            project: ProjectInfo::untitled(),
            config,
            assets: AssetRegistry::default(),
            active_scene: Scene::new("Main Scene"),
            prefabs: PrefabRegistry::default(),
            flow: GameFlow::default(),
            playtest: PlaytestState::default(),
        }
    }

    pub fn open_project(&mut self, project: ProjectInfo) {
        self.project = project;
        self.assets = AssetRegistry::default();
        self.active_scene = Scene::new("Main Scene");
        self.prefabs = PrefabRegistry::default();
        self.flow = GameFlow::default();
        self.playtest = PlaytestState::default();
    }

    pub fn project_document(&self) -> ProjectDocument {
        ProjectDocument::from_engine(self)
    }

    pub fn save_project_document(&self, path: impl AsRef<Path>) -> Result<(), ProjectIoError> {
        self.project_document().save_to_path(path)
    }

    pub fn load_project_document(&mut self, path: impl AsRef<Path>) -> Result<(), ProjectIoError> {
        let document = ProjectDocument::load_from_path(path)?;
        document.apply_to_engine(self);
        Ok(())
    }

    pub fn install_plugin(&mut self, plugin: &mut dyn Plugin) {
        let mut context = EngineContext::new(self.config.app_name.clone());
        plugin.register(&mut context);
    }

    pub fn tick(&mut self, delta_seconds: f32) -> Result<FrameStep, EngineTickError> {
        self.tick_with_input(delta_seconds, &InputState::default())
    }

    pub fn tick_with_input(
        &mut self,
        delta_seconds: f32,
        input: &InputState,
    ) -> Result<FrameStep, EngineTickError> {
        run_scene_systems(&mut self.active_scene, input, delta_seconds)
    }
}

#[cfg(test)]
mod tests {
    use crab2d_platform::{InputState, KeyCode, PlatformEvent};
    use crab2d_scene::{
        Collider2DComponent, PlayerControllerComponent, Transform2D, Vec2, Velocity2DComponent,
    };

    use crate::{Engine, EngineConfig, EngineTickError};

    #[test]
    fn engine_tick_moves_velocity_components() {
        let mut engine = Engine::new(EngineConfig::new("Crab2D Test"));
        let player = engine
            .active_scene
            .spawn_node_with_transform("Player", Transform2D::from_position(Vec2::new(4.0, 5.0)))
            .expect("player should spawn");
        engine
            .active_scene
            .add_velocity(player, Velocity2DComponent::from_xy(6.0, 8.0))
            .expect("velocity should attach");

        let frame = engine.tick(0.5).expect("tick should succeed");

        assert_eq!(frame.moved_entities, 1);
        assert_eq!(
            engine
                .active_scene
                .node(player)
                .expect("player exists")
                .transform
                .position,
            Vec2::new(7.0, 9.0)
        );
    }

    #[test]
    fn engine_tick_reports_collisions() {
        let mut engine = Engine::new(EngineConfig::new("Crab2D Test"));
        let player = engine.active_scene.spawn_node("Player");
        let crate_entity = engine
            .active_scene
            .spawn_node_with_transform("Crate", Transform2D::from_position(Vec2::new(8.0, 0.0)))
            .expect("crate should spawn");
        let collider = Collider2DComponent::rectangle(Vec2::new(16.0, 16.0));
        engine
            .active_scene
            .add_collider(player, collider)
            .expect("collider should attach");
        engine
            .active_scene
            .add_collider(crate_entity, collider)
            .expect("collider should attach");

        let frame = engine.tick(0.0).expect("tick should succeed");

        assert_eq!(frame.collisions.len(), 1);
        assert_eq!(frame.collisions[0].a, player);
        assert_eq!(frame.collisions[0].b, crate_entity);
    }

    #[test]
    fn engine_tick_rejects_invalid_delta() {
        let mut engine = Engine::new(EngineConfig::new("Crab2D Test"));

        let result = engine.tick(-1.0);

        assert_eq!(result, Err(EngineTickError::InvalidDelta));
    }

    #[test]
    fn engine_tick_with_input_drives_player_controller() {
        let mut engine = Engine::new(EngineConfig::new("Crab2D Test"));
        let player = engine.active_scene.spawn_node("Player");
        engine
            .active_scene
            .add_velocity(player, Velocity2DComponent::default())
            .expect("velocity should attach");
        engine
            .active_scene
            .add_player_controller(player, PlayerControllerComponent::new(10.0))
            .expect("controller should attach");
        let mut input = InputState::default();
        input.apply_event(PlatformEvent::KeyPressed(KeyCode::Character('w')));

        engine
            .tick_with_input(1.0, &input)
            .expect("tick should succeed");

        assert_eq!(
            engine
                .active_scene
                .node(player)
                .expect("player exists")
                .transform
                .position,
            Vec2::new(0.0, 10.0)
        );
    }
}
