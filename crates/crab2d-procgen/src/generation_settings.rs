use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationSettings {
    pub scene_name: String,
    pub map_width: u32,
    pub map_height: u32,
    pub tile_size: u32,
    pub seed: Option<u64>,
}

impl Default for GenerationSettings {
    fn default() -> Self {
        Self {
            scene_name: "GeneratedWorld".to_string(),
            map_width: 64,
            map_height: 48,
            tile_size: 32,
            seed: None,
        }
    }
}
