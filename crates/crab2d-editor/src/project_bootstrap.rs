use crab2d_core::{Engine, ProjectInfo};
use crab2d_scene::SceneError;

use crate::StarterSceneBuilder;

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
        StarterSceneBuilder::new(self.camera_name, self.player_name, self.world_root_name)
            .build(engine)
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
            .find_node_by_name("Player")
            .expect("player node should exist")
            .id;
        let camera = engine
            .active_scene
            .find_node_by_name("Camera2D")
            .expect("camera node should exist")
            .id;

        assert!(engine.active_scene.sprite(player).is_some());
        assert!(engine.active_scene.camera(camera).is_some());
        assert_eq!(
            engine
                .active_scene
                .find_node_by_tag("world")
                .expect("world root should exist")
                .name,
            "WorldRoot"
        );
    }
}
