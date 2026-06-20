use crab2d_core::{Engine, ProjectInfo};
use crab2d_scene::{Camera2DComponent, SceneError, SpriteComponent, TagComponent};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectBootstrap {
    pub project_name: String,
    pub camera_name: String,
    pub player_name: String,
    pub world_root_name: String,
}

impl ProjectBootstrap {
    pub fn empty_project(project_name: impl Into<String>) -> Self {
        Self {
            project_name: project_name.into(),
            camera_name: "Camera2D".to_string(),
            player_name: "Player".to_string(),
            world_root_name: "WorldRoot".to_string(),
        }
    }

    pub fn apply(self, engine: &mut Engine) -> Result<(), SceneError> {
        engine.open_project(ProjectInfo::new(self.project_name));

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

#[cfg(test)]
mod tests {
    use crab2d_core::{Engine, EngineConfig};

    use crate::ProjectBootstrap;

    #[test]
    fn starter_project_attaches_default_components() {
        let mut engine = Engine::new(EngineConfig::new("Crab2D Test"));

        ProjectBootstrap::empty_project("Test Project")
            .apply(&mut engine)
            .expect("bootstrap should succeed");

        let player = engine
            .active_scene
            .nodes()
            .iter()
            .find(|node| node.name == "Player")
            .expect("player node should exist")
            .id;
        let camera = engine
            .active_scene
            .nodes()
            .iter()
            .find(|node| node.name == "Camera2D")
            .expect("camera node should exist")
            .id;
        let world = engine
            .active_scene
            .nodes()
            .iter()
            .find(|node| node.name == "WorldRoot")
            .expect("world root node should exist")
            .id;

        assert!(engine.active_scene.sprite(player).is_some());
        assert!(engine.active_scene.camera(camera).is_some());
        assert_eq!(
            engine.active_scene.tag(world).expect("world tag").tag,
            "world"
        );
    }
}
