#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetKind {
    Sprite,
    Tilemap,
    Audio,
    Script,
    Config,
}
