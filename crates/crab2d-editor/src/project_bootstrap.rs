use crab2d_core::{Engine, ProjectInfo};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectBootstrap {
    pub project_name: String,
    pub default_nodes: Vec<String>,
}

impl ProjectBootstrap {
    pub fn empty_project(project_name: impl Into<String>) -> Self {
        Self {
            project_name: project_name.into(),
            default_nodes: vec![
                "Camera2D".to_string(),
                "Player".to_string(),
                "ProceduralWorld".to_string(),
            ],
        }
    }

    pub fn apply(self, engine: &mut Engine) {
        engine.open_project(ProjectInfo::new(self.project_name));

        for node_name in self.default_nodes {
            engine.active_scene.spawn_node(node_name);
        }
    }
}
