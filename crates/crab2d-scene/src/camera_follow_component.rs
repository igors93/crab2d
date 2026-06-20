use serde::{Deserialize, Serialize};

use crate::EntityId;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CameraFollowComponent {
    pub target: EntityId,
    pub smoothing: f32,
    pub enabled: bool,
}

impl CameraFollowComponent {
    pub const fn new(target: EntityId) -> Self {
        Self {
            target,
            smoothing: 0.0,
            enabled: true,
        }
    }

    pub const fn with_smoothing(mut self, smoothing: f32) -> Self {
        self.smoothing = smoothing;
        self
    }

    pub const fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    pub fn is_valid(self) -> bool {
        self.smoothing.is_finite() && self.smoothing >= 0.0
    }
}
