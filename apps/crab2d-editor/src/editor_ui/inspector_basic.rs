use super::*;

impl Crab2DEditorUi {
    pub(super) fn show_node_inspector(
        &mut self,
        ui: &mut egui::Ui,
        entity: EntityId,
        current_name: &str,
    ) {
        widgets::inspector_section(ui, "Node", true, |ui| {
            widgets::property_row(ui, "Name", |ui| {
                let response = ui.add_sized(
                    [ui.available_width(), 24.0],
                    egui::TextEdit::singleline(&mut self.name_edit),
                );
                let enter = ui.input(|input| input.key_pressed(egui::Key::Enter));
                if response.lost_focus() || (response.has_focus() && enter) {
                    let name = self.name_edit.trim().to_owned();
                    if !name.is_empty() && name != current_name {
                        match self
                            .app
                            .execute_command_with_history(EditorCommand::rename_node(entity, &name))
                        {
                            Ok(_) => self.set_success("Node renamed"),
                            Err(error) => self.set_error(format!("{error}")),
                        }
                    }
                    self.sync_selected_buffers();
                }
            });
            widgets::property_row(ui, "Type", |ui| {
                ui.label(self.node_type(entity));
            });
        });
    }

    pub(super) fn show_transform_inspector(
        &mut self,
        ui: &mut egui::Ui,
        entity: EntityId,
        transform_before: Transform2D,
    ) {
        let mut transform = transform_before;
        widgets::inspector_section(ui, "Transform2D", true, |ui| {
            let mut changed = false;
            let mut drag_started = false;
            let mut drag_stopped = false;
            let t = theme();

            egui::Grid::new("transform_editor")
                .num_columns(2)
                .spacing([8.0, 6.0])
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("Position")
                            .color(t.colors.text_secondary)
                            .size(12.0),
                    );
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 3.0;
                        ui.label(
                            egui::RichText::new("X")
                                .size(10.5)
                                .color(t.colors.axis_x)
                                .strong(),
                        );
                        drag_value(
                            ui,
                            &mut transform.position.x,
                            1.0,
                            &mut changed,
                            &mut drag_started,
                            &mut drag_stopped,
                        );
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new("Y")
                                .size(10.5)
                                .color(t.colors.axis_y)
                                .strong(),
                        );
                        drag_value(
                            ui,
                            &mut transform.position.y,
                            1.0,
                            &mut changed,
                            &mut drag_started,
                            &mut drag_stopped,
                        );
                    });
                    ui.end_row();

                    ui.label(
                        egui::RichText::new("Scale")
                            .color(t.colors.text_secondary)
                            .size(12.0),
                    );
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 3.0;
                        ui.label(
                            egui::RichText::new("X")
                                .size(10.5)
                                .color(t.colors.axis_x)
                                .strong(),
                        );
                        drag_value(
                            ui,
                            &mut transform.scale.x,
                            0.05,
                            &mut changed,
                            &mut drag_started,
                            &mut drag_stopped,
                        );
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new("Y")
                                .size(10.5)
                                .color(t.colors.axis_y)
                                .strong(),
                        );
                        drag_value(
                            ui,
                            &mut transform.scale.y,
                            0.05,
                            &mut changed,
                            &mut drag_started,
                            &mut drag_stopped,
                        );
                    });
                    ui.end_row();

                    ui.label(
                        egui::RichText::new("Rotation")
                            .color(t.colors.text_secondary)
                            .size(12.0),
                    );
                    ui.horizontal(|ui| {
                        let mut degrees = transform.rotation_radians.to_degrees();
                        let response = ui.add_sized(
                            [t.sizing.property_input_width, 22.0],
                            egui::DragValue::new(&mut degrees).speed(1.0).suffix("°"),
                        );
                        changed |= response.changed();
                        drag_started |= response.drag_started();
                        drag_stopped |= response.drag_stopped();
                        transform.rotation_radians = degrees.to_radians();
                    });
                    ui.end_row();
                });

            if drag_started {
                self.transform_drag = Some((entity, transform_before));
            }

            if changed {
                if self.transform_drag.is_some() {
                    if let Err(error) = self
                        .app
                        .execute_command(EditorCommand::move_node(entity, transform))
                    {
                        self.set_error(format!("{error}"));
                    }
                } else if let Err(error) = self
                    .app
                    .execute_command_with_history(EditorCommand::move_node(entity, transform))
                {
                    self.set_error(format!("{error}"));
                } else {
                    self.status = "Transform updated".to_owned();
                    self.status_tone = StatusTone::Info;
                }
            }

            if drag_stopped {
                if let Some((eid, before)) = self.transform_drag.take() {
                    let after = self.app.node_transform(eid).unwrap_or(transform);
                    self.app.record_move_node(eid, before, after);
                    self.status = "Transform updated".to_owned();
                    self.status_tone = StatusTone::Info;
                }
            }
        });
    }

    pub(super) fn create_preset_node(&mut self, preset: GameplayPreset) {
        match self.app.create_preset_node(preset) {
            Ok(entity) => {
                self.select_node(entity);
                self.set_success(format!("{} preset created", preset.label()));
            }
            Err(error) => self.set_error(format!("{error}")),
        }
    }
}
