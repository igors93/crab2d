use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioComponent {
    pub clip_path: String,
    pub volume: f32,
    pub looping: bool,
    pub auto_play: bool,
    pub spatial: bool,
}

impl AudioComponent {
    pub fn new(clip_path: impl Into<String>) -> Self {
        Self {
            clip_path: clip_path.into(),
            volume: 1.0,
            looping: false,
            auto_play: false,
            spatial: false,
        }
    }
    pub fn looping(mut self) -> Self {
        self.looping = true;
        self
    }
    pub fn auto_play(mut self) -> Self {
        self.auto_play = true;
        self
    }
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }
}
