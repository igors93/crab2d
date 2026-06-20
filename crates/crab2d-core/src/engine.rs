use crab2d_assets::AssetRegistry;
use crab2d_plugin_api::{EngineContext, Plugin};
use crab2d_scene::Scene;

use crate::{EngineConfig, ProjectInfo};

#[derive(Debug)]
pub struct Engine {
    pub config: EngineConfig,
    pub project: ProjectInfo,
    pub assets: AssetRegistry,
    pub active_scene: Scene,
}

impl Engine {
    pub fn new(config: EngineConfig) -> Self {
        Self {
            project: ProjectInfo::untitled(),
            config,
            assets: AssetRegistry::default(),
            active_scene: Scene::new("Main Scene"),
        }
    }

    pub fn open_project(&mut self, project: ProjectInfo) {
        self.project = project;
        self.active_scene = Scene::new("Main Scene");
    }

    pub fn install_plugin(&mut self, plugin: &mut dyn Plugin) {
        let mut context = EngineContext::new(self.config.app_name.clone());
        plugin.register(&mut context);
    }

    pub fn tick(&mut self, _delta_seconds: f32) {
        // Runtime systems will be scheduled here as the engine grows.
    }
}
