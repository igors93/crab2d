use crab2d_assets::AssetRegistry;
use crab2d_plugin_api::{EngineContext, Plugin};
use crab2d_scene::Scene;

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub app_name: String,
    pub target_fps: u32,
}

impl EngineConfig {
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
            target_fps: 60,
        }
    }
}

#[derive(Debug)]
pub struct Engine {
    pub config: EngineConfig,
    pub assets: AssetRegistry,
    pub active_scene: Scene,
}

impl Engine {
    pub fn new(config: EngineConfig) -> Self {
        Self {
            config,
            assets: AssetRegistry::default(),
            active_scene: Scene::new("Main Scene"),
        }
    }

    pub fn install_plugin(&mut self, plugin: &mut dyn Plugin) {
        let mut context = EngineContext::new(self.config.app_name.clone());
        plugin.register(&mut context);
    }

    pub fn tick(&mut self, _delta_seconds: f32) {
        // Future runtime systems will update scenes, physics, animation, and scripts here.
    }
}
