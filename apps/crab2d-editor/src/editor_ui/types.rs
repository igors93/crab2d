use super::*;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct NodeView {
    pub(super) id: EntityId,
    pub(super) name: String,
    pub(super) transform: Transform2D,
    pub(super) sprite_path: Option<String>,
    pub(super) camera: Option<Camera2DComponent>,
    pub(super) tilemap: Option<TilemapComponent>,
    pub(super) collider: Option<Collider2DComponent>,
    pub(super) trigger: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EditorTool {
    Select,
    Pan,
    TileBrush,
    EraseTile,
}

impl EditorTool {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Select => "Select",
            Self::Pan => "Pan",
            Self::TileBrush => "Tile Brush",
            Self::EraseTile => "Erase",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LeftPanelTab {
    Scene,
    Layers,
    Library,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BottomDockTab {
    TilePalette,
    Assets,
    Output,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AssetBrowserTab {
    Images,
    Broken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AssetCategory {
    All,
    Sprites,
    Tilemaps,
    Ui,
    Other,
}

impl AssetCategory {
    pub(super) const ALL: [Self; 5] = [
        Self::All,
        Self::Sprites,
        Self::Tilemaps,
        Self::Ui,
        Self::Other,
    ];

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Sprites => "Sprites",
            Self::Tilemaps => "Tilemaps",
            Self::Ui => "UI",
            Self::Other => "Other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum DockPanel {
    Scene,
    Inspector,
    Assets,
}

impl DockPanel {
    pub(super) const ALL: [Self; 3] = [Self::Scene, Self::Assets, Self::Inspector];

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Scene => "Scene",
            Self::Inspector => "Inspector",
            Self::Assets => "Assets",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DockSlot {
    Left,
    Right,
    Bottom,
    Hidden,
}

impl DockSlot {
    pub(super) const ALL: [Self; 4] = [Self::Left, Self::Right, Self::Bottom, Self::Hidden];

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Bottom => "Bottom",
            Self::Hidden => "Hidden",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ViewportScaleDrag {
    pub(super) entity: EntityId,
    pub(super) before: Transform2D,
    pub(super) start_pointer: egui::Pos2,
    pub(super) start_size: egui::Vec2,
    pub(super) handle: ResizeHandle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum ViewportDrag {
    Move {
        entity: EntityId,
        before: Transform2D,
    },
    Scale(ViewportScaleDrag),
}

impl ViewportDrag {
    pub(super) fn entity(self) -> EntityId {
        match self {
            Self::Move { entity, .. } => entity,
            Self::Scale(drag) => drag.entity,
        }
    }

    pub(super) fn before(self) -> Transform2D {
        match self {
            Self::Move { before, .. } => before,
            Self::Scale(drag) => drag.before,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ResizeHandle {
    pub(super) horizontal: f32,
    pub(super) vertical: f32,
}

impl ResizeHandle {
    pub(super) const TOP_LEFT: Self = Self {
        horizontal: -1.0,
        vertical: -1.0,
    };
    pub(super) const TOP_RIGHT: Self = Self {
        horizontal: 1.0,
        vertical: -1.0,
    };
    pub(super) const BOTTOM_LEFT: Self = Self {
        horizontal: -1.0,
        vertical: 1.0,
    };
    pub(super) const BOTTOM_RIGHT: Self = Self {
        horizontal: 1.0,
        vertical: 1.0,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ProjectDialog {
    NewProject,
    OpenProject,
    SaveAs,
    SaveBeforePlay,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AssetCardResult {
    pub(super) clicked: bool,
    pub(super) load_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AssetPlacementDrag {
    pub(super) asset_path: String,
    pub(super) display_name: String,
}
