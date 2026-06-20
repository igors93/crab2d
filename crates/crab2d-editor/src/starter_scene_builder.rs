use crab2d_core::Engine;
use crab2d_scene::{Camera2DComponent, SceneError, SpriteComponent, TagComponent};

pub struct StarterSceneBuilder {
    camera_name: String,
    player_name: String,
    world_root_name: String,
}

impl StarterSceneBuilder {
    pub fn new(
        camera_name: impl Into<String>,
        player_name: impl Into<String>,
        world_root_name: impl Into<String>,
    ) -> Self {
        Self {
            camera_name: camera_name.into(),
            player_name: player_name.into(),
            world_root_name: world_root_name.into(),
        }
    }

    pub fn build(self, engine: &mut Engine) -> Result<(), SceneError> {
        let camera = engine.active_scene.spawn_node(self.camera_name);
        engine
            .active_scene
            .add_camera(camera, Camera2DComponent::default())?;

        let player = engine.active_scene.spawn_node(self.player_name);
        engine
            .active_scene
            .add_tag(player, TagComponent::new("player"))?;
        engine
            .active_scene
            .add_sprite(player, SpriteComponent::new("sprites/player.png"))?;

        let world_root = engine.active_scene.spawn_node(self.world_root_name);
        engine
            .active_scene
            .add_tag(world_root, TagComponent::new("world"))?;

        Ok(())
    }
}
