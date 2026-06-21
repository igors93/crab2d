use super::*;

impl Crab2DEditorUi {
    pub(super) fn node_label(&self, id: EntityId) -> String {
        let name = self
            .app
            .find_node(id)
            .map(|node| node.name.as_str())
            .unwrap_or("?");

        let (kind, _) = self.node_kind(id);
        format!("{kind} {name}")
    }

    pub(super) fn node_kind(&self, id: EntityId) -> (&'static str, StatusTone) {
        if self.app.node_tilemap(id).is_some() {
            ("MAP", StatusTone::Info)
        } else if self.app.node_camera(id).is_some() {
            ("CAM", StatusTone::Warning)
        } else if self.app.node_player_controller(id).is_some() {
            ("PLY", StatusTone::Success)
        } else if self.app.node_trigger(id).is_some() {
            ("TRG", StatusTone::Warning)
        } else if self.app.node_sprite(id).is_some() {
            ("SPR", StatusTone::Success)
        } else if self.app.node_tag(id).is_some() {
            ("TAG", StatusTone::Info)
        } else {
            ("NOD", StatusTone::Info)
        }
    }

    pub(super) fn node_type(&self, id: EntityId) -> &'static str {
        if self.app.node_tilemap(id).is_some() {
            "Tilemap"
        } else if self.app.node_camera(id).is_some() {
            "Camera2D"
        } else if self.app.node_player_controller(id).is_some() {
            "Player"
        } else if self.app.node_trigger(id).is_some() {
            "Trigger"
        } else if self.app.node_sprite(id).is_some() {
            "Sprite2D"
        } else {
            "Node2D"
        }
    }

    pub(super) fn node_color(&self, item: &NodeView) -> egui::Color32 {
        let theme = theme();
        if self.selected == Some(item.id) {
            theme.colors.accent
        } else if item.camera.is_some() {
            theme.colors.camera
        } else if item.sprite_path.is_some() {
            theme.colors.success
        } else if item.tilemap.is_some() {
            theme.colors.accent_soft
        } else {
            theme.colors.warning
        }
    }
}
