#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum EditorMode {
    #[default]
    Select,
    TileBrush,
    CollisionEdit,
    ProceduralPreview,
}
