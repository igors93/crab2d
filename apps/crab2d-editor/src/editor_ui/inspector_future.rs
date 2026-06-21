use super::*;

impl Crab2DEditorUi {
    pub(super) fn show_behavior_inspector(&mut self, ui: &mut egui::Ui, entity: EntityId) {
        let has_behavior = self.app.find_node(entity).and(None::<bool>).is_some();
        let _ = has_behavior;
        widgets::inspector_section(ui, "Behavior (Script)", false, |ui| {
            widgets::property_row(ui, "Script Path", |ui| {
                ui.add_sized(
                    [ui.available_width().max(80.0), 24.0],
                    egui::TextEdit::singleline(&mut self.behavior_script_edit),
                );
            });
            widgets::property_row(ui, "Enabled", |ui| {
                ui.checkbox(&mut self.behavior_enabled_edit, "");
            });
            if widgets::toolbar_button(
                ui,
                "+ Add Behavior",
                "Add behavior/script component",
                true,
                false,
            )
            .clicked()
            {
                self.set_status("Behavior component support requires scene API extension");
            }
        });
    }

    pub(super) fn show_audio_inspector(&mut self, ui: &mut egui::Ui, entity: EntityId) {
        let _ = entity;
        widgets::inspector_section(ui, "Audio", false, |ui| {
            widgets::property_row(ui, "Clip Path", |ui| {
                ui.add_sized(
                    [ui.available_width().max(80.0), 24.0],
                    egui::TextEdit::singleline(&mut self.audio_clip_edit),
                );
            });
            widgets::property_row(ui, "Volume", |ui| {
                ui.add_sized(
                    [theme().sizing.property_input_width, 22.0],
                    egui::DragValue::new(&mut self.audio_volume_edit)
                        .speed(0.05)
                        .range(0.0..=1.0),
                );
            });
            widgets::property_row(ui, "Looping", |ui| {
                ui.checkbox(&mut self.audio_looping_edit, "");
            });
            if widgets::toolbar_button(ui, "+ Add Audio", "Add audio component", true, false)
                .clicked()
            {
                self.set_status("Audio component support requires scene API extension");
            }
        });
    }

    pub(super) fn show_animation_inspector(&mut self, ui: &mut egui::Ui, entity: EntityId) {
        let _ = entity;
        widgets::inspector_section(ui, "Animation", false, |ui| {
            widgets::property_row(ui, "Spritesheet", |ui| {
                ui.add_sized(
                    [ui.available_width().max(80.0), 24.0],
                    egui::TextEdit::singleline(&mut self.animation_spritesheet_edit),
                );
            });
            if widgets::toolbar_button(
                ui,
                "+ Add Animation",
                "Add animation component",
                true,
                false,
            )
            .clicked()
            {
                self.set_status("Animation component support requires scene API extension");
            }
        });
    }

    pub(super) fn show_ui_label_inspector(&mut self, ui: &mut egui::Ui, entity: EntityId) {
        let _ = entity;
        widgets::inspector_section(ui, "UI Label", false, |ui| {
            widgets::property_row(ui, "Text", |ui| {
                ui.add_sized(
                    [ui.available_width().max(80.0), 24.0],
                    egui::TextEdit::singleline(&mut self.ui_label_text_edit),
                );
            });
            widgets::property_row(ui, "Font Size", |ui| {
                ui.add_sized(
                    [theme().sizing.property_input_width, 22.0],
                    egui::DragValue::new(&mut self.ui_label_font_size_edit)
                        .speed(1.0)
                        .range(6.0..=128.0),
                );
            });
            if widgets::toolbar_button(ui, "+ Add UI Label", "Add UI label component", true, false)
                .clicked()
            {
                self.set_status("UI label component support requires scene API extension");
            }
        });
    }

    pub(super) fn show_particle_inspector(&mut self, ui: &mut egui::Ui, entity: EntityId) {
        let _ = entity;
        widgets::inspector_section(ui, "Particle Emitter", false, |ui| {
            widgets::property_row(ui, "Texture", |ui| {
                ui.add_sized(
                    [ui.available_width().max(80.0), 24.0],
                    egui::TextEdit::singleline(&mut self.particle_texture_edit),
                );
            });
            widgets::property_row(ui, "Emit Rate", |ui| {
                ui.add_sized(
                    [theme().sizing.property_input_width, 22.0],
                    egui::DragValue::new(&mut self.particle_emit_rate_edit)
                        .speed(1.0)
                        .range(0.0..=1000.0),
                );
            });
            if widgets::toolbar_button(
                ui,
                "+ Add Particles",
                "Add particle emitter component",
                true,
                false,
            )
            .clicked()
            {
                self.set_status("Particle emitter component support requires scene API extension");
            }
        });
    }
}
