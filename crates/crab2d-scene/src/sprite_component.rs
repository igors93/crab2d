use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteComponent {
    #[serde(alias = "asset_path")]
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

#[cfg(test)]
mod tests {
    use super::SpriteComponent;

    #[test]
    fn sprite_component_reads_legacy_asset_path_but_writes_sprite_path() {
        let legacy_json = r#"{
            "asset_path": "sprites/player.png",
            "visible": true,
            "z_index": 3
        }"#;

        let component: SpriteComponent =
            serde_json::from_str(legacy_json).expect("legacy sprite should deserialize");

        assert_eq!(component.sprite_path, "sprites/player.png");
        assert!(component.visible);
        assert_eq!(component.z_index, 3);

        let serialized = serde_json::to_value(&component).expect("sprite should serialize to json");

        assert_eq!(serialized["sprite_path"], "sprites/player.png");
        assert!(serialized.get("asset_path").is_none());
    }
}
