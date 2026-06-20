use crab2d_core::Engine;
use crab2d_scene::{
    Camera2DComponent, Collider2DComponent, SceneError, SpriteComponent, TagComponent, Vec2,
    Velocity2DComponent,
};

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
        self.validate_names()?;

        let camera = engine.active_scene.try_spawn_node(self.camera_name)?;
        engine
            .active_scene
            .add_camera(camera, Camera2DComponent::default())?;

        let player = engine.active_scene.try_spawn_node(self.player_name)?;
        engine
            .active_scene
            .add_tag(player, TagComponent::new("player"))?;
        engine
            .active_scene
            .add_sprite(player, SpriteComponent::new("sprites/player.png"))?;
        engine
            .active_scene
            .add_velocity(player, Velocity2DComponent::default())?;
        engine.active_scene.add_collider(
            player,
            Collider2DComponent::rectangle(Vec2::new(24.0, 24.0)),
        )?;

        let world_root = engine.active_scene.try_spawn_node(self.world_root_name)?;
        engine
            .active_scene
            .add_tag(world_root, TagComponent::new("world"))?;

        Ok(())
    }

    fn validate_names(&self) -> Result<(), SceneError> {
        if self.camera_name.is_empty()
            || self.player_name.is_empty()
            || self.world_root_name.is_empty()
        {
            return Err(SceneError::EmptyNodeName);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crab2d_core::{Engine, EngineConfig};
    use crab2d_scene::SceneError;

    use crate::StarterSceneBuilder;

    #[test]
    fn starter_scene_builder_returns_error_for_empty_camera_name() {
        assert_empty_name_is_rejected(StarterSceneBuilder::new("", "Player", "WorldRoot"));
    }

    #[test]
    fn starter_scene_builder_returns_error_for_empty_player_name() {
        assert_empty_name_is_rejected(StarterSceneBuilder::new("Camera2D", "", "WorldRoot"));
    }

    #[test]
    fn starter_scene_builder_returns_error_for_empty_world_root_name() {
        assert_empty_name_is_rejected(StarterSceneBuilder::new("Camera2D", "Player", ""));
    }

    fn assert_empty_name_is_rejected(builder: StarterSceneBuilder) {
        let mut engine = Engine::new(EngineConfig::new("Crab2D Test"));

        let result = builder.build(&mut engine);

        assert_eq!(result, Err(SceneError::EmptyNodeName));
        assert!(engine.active_scene.is_empty());
    }
}
