use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteComponent {
    pub sprite_path: String,
    pub visible: bool,
    pub z_index: i32,
}

impl SpriteComponent {
    pub fn new(sprite_path: impl Into<String>) -> Self {
        Self {
            sprite_path: sprite_path.into(),
            visible: true,
            z_index: 0,
        }
    }

    pub fn hidden(mut self) -> Self {
        self.visible = false;
        self
    }

    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }
}
