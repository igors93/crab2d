use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetKind {
    Sprite,
    Tilemap,
    Audio,
    Script,
    Config,
}
