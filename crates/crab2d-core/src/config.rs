#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineConfig {
    pub app_name: String,
    pub target_fps: u32,
}

impl EngineConfig {
    pub const DEFAULT_TARGET_FPS: u32 = 60;

    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
            target_fps: Self::DEFAULT_TARGET_FPS,
        }
    }

    pub fn with_target_fps(mut self, target_fps: u32) -> Self {
        self.target_fps = target_fps.max(1);
        self
    }
}
