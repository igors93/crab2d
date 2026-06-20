use crate::Vec2;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform2D {
    pub position: Vec2,
    pub rotation_radians: f32,
    pub scale: Vec2,
}

impl Transform2D {
    pub const IDENTITY: Self = Self {
        position: Vec2::ZERO,
        rotation_radians: 0.0,
        scale: Vec2::ONE,
    };

    pub const fn from_position(position: Vec2) -> Self {
        Self {
            position,
            rotation_radians: 0.0,
            scale: Vec2::ONE,
        }
    }

    pub fn is_finite(self) -> bool {
        self.position.is_finite() && self.rotation_radians.is_finite() && self.scale.is_finite()
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::IDENTITY
    }
}
