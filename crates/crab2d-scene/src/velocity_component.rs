use serde::{Deserialize, Serialize};

use crate::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Velocity2DComponent {
    pub linear: Vec2,
}

impl Velocity2DComponent {
    pub const ZERO: Self = Self { linear: Vec2::ZERO };

    pub const fn new(linear: Vec2) -> Self {
        Self { linear }
    }

    pub const fn from_xy(x: f32, y: f32) -> Self {
        Self {
            linear: Vec2::new(x, y),
        }
    }

    pub fn is_finite(self) -> bool {
        self.linear.is_finite()
    }
}

impl Default for Velocity2DComponent {
    fn default() -> Self {
        Self::ZERO
    }
}
