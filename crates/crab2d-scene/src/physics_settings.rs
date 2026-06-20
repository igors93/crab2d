use crate::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhysicsSettings {
    pub gravity: Vec2,
    pub terminal_velocity: f32,
    pub enabled: bool,
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        Self {
            gravity: Vec2::new(0.0, -980.0), // pixels/sec^2 downward
            terminal_velocity: 1200.0,
            enabled: false, // disabled by default so top-down games aren't affected
        }
    }
}
