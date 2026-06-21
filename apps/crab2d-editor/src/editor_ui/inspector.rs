use super::*;

impl Crab2DEditorUi {
    pub(super) fn show_inspector_contents(&mut self, ui: &mut egui::Ui) {
        let theme = theme();
        widgets::panel_header(ui, "Inspector", None);
        ui.add_space(theme.spacing.sm);

        let Some(entity) = self.selected else {
            widgets::inset_frame().show(ui, |ui| {
                ui.label(egui::RichText::new("No node selected").color(theme.colors.text_muted));
            });
            return;
        };

        let Some(node) = self.app.find_node(entity).cloned() else {
            self.selected = None;
            return;
        };

        self.show_selected_node_header(ui, entity, node.name.as_str());
        ui.add_sized(
            [ui.available_width(), 26.0],
            egui::TextEdit::singleline(&mut self.inspector_filter_edit)
                .hint_text("Filter properties..."),
        );

        egui::ScrollArea::vertical()
            .id_salt("inspector_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if self.inspector_allows("Node") {
                    self.show_node_inspector(ui, entity, node.name.as_str());
                }
                if self.inspector_allows("Transform2D") {
                    self.show_transform_inspector(ui, entity, node.transform);
                }
                if self.inspector_allows("Tag") {
                    self.show_tag_inspector(ui, entity);
                }
                if self.inspector_allows("Sprite") {
                    self.show_sprite_inspector(ui, entity);
                }
                if self.inspector_allows("Tilemap") {
                    self.show_tilemap_inspector(ui, entity);
                }
                if self.inspector_allows("Camera2D") {
                    self.show_camera_inspector(ui, entity);
                }
                if self.inspector_allows("Velocity2D") {
                    self.show_velocity_inspector(ui, entity);
                }
                if self.inspector_allows("Collider2D") {
                    self.show_collider_inspector(ui, entity);
                }
                if self.inspector_allows("PlayerController") {
                    self.show_player_controller_inspector(ui, entity);
                }
                if self.inspector_allows("CameraFollow") {
                    self.show_camera_follow_inspector(ui, entity);
                }
                if self.inspector_allows("Trigger") {
                    self.show_trigger_inspector(ui, entity);
                }
                if self.inspector_allows("Behavior") {
                    self.show_behavior_inspector(ui, entity);
                }
                if self.inspector_allows("Audio") {
                    self.show_audio_inspector(ui, entity);
                }
                if self.inspector_allows("Animation") {
                    self.show_animation_inspector(ui, entity);
                }
                if self.inspector_allows("UI Label") {
                    self.show_ui_label_inspector(ui, entity);
                }
                if self.inspector_allows("Particle") {
                    self.show_particle_inspector(ui, entity);
                }

                ui.add_space(theme.spacing.md);
                self.show_add_component_panel(ui, entity);
                ui.add_space(theme.spacing.sm);
            });
    }

    fn inspector_allows(&self, label: &str) -> bool {
        let filter = self.inspector_filter_edit.trim().to_lowercase();
        filter.is_empty() || label.to_lowercase().contains(filter.as_str())
    }
}
