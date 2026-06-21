use super::*;

impl Crab2DEditorUi {
    pub(super) fn show_velocity_inspector(&mut self, ui: &mut egui::Ui, entity: EntityId) {
        if self.app.node_velocity(entity).is_none() {
            widgets::inspector_section(ui, "Velocity2D", false, |ui| {
                if widgets::toolbar_button(
                    ui,
                    "+ Add Velocity",
                    "Add velocity component",
                    true,
                    false,
                )
                .clicked()
                {
                    match self
                        .app
                        .execute_command_with_history(EditorCommand::attach_velocity(
                            entity,
                            Velocity2DComponent::ZERO,
                        )) {
                        Ok(_) => self.set_success("Velocity added"),
                        Err(error) => self.set_error(format!("{error}")),
                    }
                }
            });
            return;
        }

        widgets::inspector_section(ui, "Velocity2D", false, |ui| {
            widgets::property_row(ui, "Linear", |ui| {
                ui.add_sized(
                    [theme().sizing.property_input_width, 22.0],
                    egui::DragValue::new(&mut self.velocity_x_edit).speed(1.0),
                );
                ui.add_sized(
                    [theme().sizing.property_input_width, 22.0],
                    egui::DragValue::new(&mut self.velocity_y_edit).speed(1.0),
                );
            });
            widgets::property_row(ui, "Action", |ui| {
                if widgets::toolbar_button(ui, "Apply", "Apply velocity", true, false).clicked() {
                    let velocity = Velocity2DComponent::new(Vec2::new(
                        self.velocity_x_edit,
                        self.velocity_y_edit,
                    ));
                    match self
                        .app
                        .execute_command_with_history(EditorCommand::attach_velocity(
                            entity, velocity,
                        )) {
                        Ok(_) => self.set_success("Velocity updated"),
                        Err(error) => self.set_error(format!("{error}")),
                    }
                }
            });
            widgets::property_row(ui, "Remove", |ui| {
                if widgets::toolbar_button(ui, "Remove Component", "Remove velocity", true, false)
                    .clicked()
                {
                    self.remove_component(
                        entity,
                        EditorComponentKind::Velocity,
                        "Velocity removed",
                    );
                }
            });
        });
    }

    pub(super) fn show_collider_inspector(&mut self, ui: &mut egui::Ui, entity: EntityId) {
        if self.app.node_collider(entity).is_none() {
            widgets::inspector_section(ui, "Collider2D", false, |ui| {
                if widgets::toolbar_button(
                    ui,
                    "+ Add Collider2D",
                    "Add collider component",
                    true,
                    false,
                )
                .clicked()
                {
                    let collider = Collider2DComponent::rectangle(Vec2::new(24.0, 24.0));
                    match self
                        .app
                        .execute_command_with_history(EditorCommand::attach_collider(
                            entity, collider,
                        )) {
                        Ok(_) => self.set_success("Collider added"),
                        Err(error) => self.set_error(format!("{error}")),
                    }
                }
            });
            return;
        }

        widgets::inspector_section(ui, "Collider2D", false, |ui| {
            widgets::property_row(ui, "Size", |ui| {
                ui.add_sized(
                    [theme().sizing.property_input_width, 22.0],
                    egui::DragValue::new(&mut self.collider_width_edit)
                        .speed(1.0)
                        .range(1.0..=4096.0),
                );
                ui.add_sized(
                    [theme().sizing.property_input_width, 22.0],
                    egui::DragValue::new(&mut self.collider_height_edit)
                        .speed(1.0)
                        .range(1.0..=4096.0),
                );
            });
            widgets::property_row(ui, "Sensor", |ui| {
                ui.checkbox(&mut self.collider_sensor_edit, "");
            });
            widgets::property_row(ui, "Action", |ui| {
                if widgets::toolbar_button(ui, "Apply", "Apply collider", true, false).clicked() {
                    let mut collider = Collider2DComponent::rectangle(Vec2::new(
                        self.collider_width_edit,
                        self.collider_height_edit,
                    ));
                    if self.collider_sensor_edit {
                        collider = collider.sensor();
                    }
                    match self
                        .app
                        .execute_command_with_history(EditorCommand::attach_collider(
                            entity, collider,
                        )) {
                        Ok(_) => self.set_success("Collider updated"),
                        Err(error) => self.set_error(format!("{error}")),
                    }
                }
            });
            widgets::property_row(ui, "Remove", |ui| {
                if widgets::toolbar_button(ui, "Remove Component", "Remove collider", true, false)
                    .clicked()
                {
                    self.remove_component(
                        entity,
                        EditorComponentKind::Collider,
                        "Collider removed",
                    );
                }
            });
        });
    }

    pub(super) fn show_player_controller_inspector(&mut self, ui: &mut egui::Ui, entity: EntityId) {
        if self.app.node_player_controller(entity).is_none() {
            widgets::inspector_section(ui, "PlayerController", false, |ui| {
                if widgets::toolbar_button(
                    ui,
                    "+ Add PlayerController",
                    "Add player controller component",
                    true,
                    false,
                )
                .clicked()
                {
                    match self.app.execute_command_with_history(
                        EditorCommand::attach_player_controller(
                            entity,
                            PlayerControllerComponent::default(),
                        ),
                    ) {
                        Ok(_) => self.set_success("Player controller added"),
                        Err(error) => self.set_error(format!("{error}")),
                    }
                }
            });
            return;
        }

        widgets::inspector_section(ui, "PlayerController", false, |ui| {
            widgets::property_row(ui, "Move Speed", |ui| {
                ui.add_sized(
                    [theme().sizing.property_input_width, 22.0],
                    egui::DragValue::new(&mut self.controller_speed_edit)
                        .speed(5.0)
                        .range(0.0..=4096.0),
                );
            });
            widgets::property_row(ui, "Enabled", |ui| {
                ui.checkbox(&mut self.controller_enabled_edit, "");
            });
            widgets::property_row(ui, "Action", |ui| {
                if widgets::toolbar_button(ui, "Apply", "Apply controller", true, false).clicked() {
                    let mut controller = PlayerControllerComponent::new(self.controller_speed_edit);
                    if !self.controller_enabled_edit {
                        controller = controller.disabled();
                    }
                    match self.app.execute_command_with_history(
                        EditorCommand::attach_player_controller(entity, controller),
                    ) {
                        Ok(_) => self.set_success("Player controller updated"),
                        Err(error) => self.set_error(format!("{error}")),
                    }
                }
            });
            widgets::property_row(ui, "Remove", |ui| {
                if widgets::toolbar_button(ui, "Remove Component", "Remove controller", true, false)
                    .clicked()
                {
                    self.remove_component(
                        entity,
                        EditorComponentKind::PlayerController,
                        "Player controller removed",
                    );
                }
            });
        });
    }

    pub(super) fn show_camera_follow_inspector(&mut self, ui: &mut egui::Ui, entity: EntityId) {
        if self.app.node_camera_follow(entity).is_none() {
            widgets::inspector_section(ui, "CameraFollow", false, |ui| {
                if widgets::toolbar_button(
                    ui,
                    "+ Add CameraFollow",
                    "Add camera follow component",
                    true,
                    false,
                )
                .clicked()
                {
                    match self.camera_follow_target_edit.trim().parse::<u64>() {
                        Ok(raw) => match self.app.execute_command_with_history(
                            EditorCommand::attach_camera_follow(
                                entity,
                                CameraFollowComponent::new(EntityId::from_raw(raw)),
                            ),
                        ) {
                            Ok(_) => self.set_success("Camera follow added"),
                            Err(error) => self.set_error(format!("{error}")),
                        },
                        Err(_) => self.set_error("Camera follow target must be an entity id"),
                    }
                }
            });
            return;
        }

        widgets::inspector_section(ui, "CameraFollow", false, |ui| {
            widgets::property_row(ui, "Target Id", |ui| {
                ui.add_sized(
                    [theme().sizing.property_input_width, 24.0],
                    egui::TextEdit::singleline(&mut self.camera_follow_target_edit),
                );
            });
            widgets::property_row(ui, "Smoothing", |ui| {
                ui.add_sized(
                    [theme().sizing.property_input_width, 22.0],
                    egui::DragValue::new(&mut self.camera_follow_smoothing_edit)
                        .speed(0.1)
                        .range(0.0..=60.0),
                );
            });
            widgets::property_row(ui, "Enabled", |ui| {
                ui.checkbox(&mut self.camera_follow_enabled_edit, "");
            });
            widgets::property_row(ui, "Lock X", |ui| {
                ui.checkbox(&mut self.camera_follow_lock_x_edit, "Freeze horizontal");
            });
            widgets::property_row(ui, "Lock Y", |ui| {
                ui.checkbox(&mut self.camera_follow_lock_y_edit, "Freeze vertical");
            });
            widgets::property_row(ui, "Dead Zone", |ui| {
                ui.add_sized(
                    [theme().sizing.property_input_width, 22.0],
                    egui::DragValue::new(&mut self.camera_follow_dead_zone_edit)
                        .speed(1.0)
                        .range(0.0..=512.0),
                );
            });
            widgets::property_row(ui, "Action", |ui| {
                if widgets::toolbar_button(ui, "Apply", "Apply camera follow", true, false)
                    .clicked()
                {
                    match self.camera_follow_target_edit.trim().parse::<u64>() {
                        Ok(raw) => {
                            let mut follow = CameraFollowComponent::new(EntityId::from_raw(raw))
                                .with_smoothing(self.camera_follow_smoothing_edit)
                                .with_dead_zone(self.camera_follow_dead_zone_edit);
                            if !self.camera_follow_enabled_edit {
                                follow = follow.disabled();
                            }
                            if self.camera_follow_lock_x_edit {
                                follow = follow.with_lock_x();
                            }
                            if self.camera_follow_lock_y_edit {
                                follow = follow.with_lock_y();
                            }
                            match self.app.execute_command_with_history(
                                EditorCommand::attach_camera_follow(entity, follow),
                            ) {
                                Ok(_) => self.set_success("Camera follow updated"),
                                Err(error) => self.set_error(format!("{error}")),
                            }
                        }
                        Err(_) => self.set_error("Camera follow target must be an entity id"),
                    }
                }
            });
            widgets::property_row(ui, "Remove", |ui| {
                if widgets::toolbar_button(ui, "Remove Component", "Remove follow", true, false)
                    .clicked()
                {
                    self.remove_component(
                        entity,
                        EditorComponentKind::CameraFollow,
                        "Camera follow removed",
                    );
                }
            });
        });
    }

    pub(super) fn show_trigger_inspector(&mut self, ui: &mut egui::Ui, entity: EntityId) {
        if self.app.node_trigger(entity).is_none() {
            widgets::inspector_section(ui, "Trigger", false, |ui| {
                if widgets::toolbar_button(
                    ui,
                    "+ Add Trigger",
                    "Add trigger component",
                    true,
                    false,
                )
                .clicked()
                {
                    let trigger = TriggerComponent::new(self.trigger_name_edit.trim());
                    match self
                        .app
                        .execute_command_with_history(EditorCommand::attach_trigger(
                            entity, trigger,
                        )) {
                        Ok(_) => self.set_success("Trigger added"),
                        Err(error) => self.set_error(format!("{error}")),
                    }
                }
            });
            return;
        }

        widgets::inspector_section(ui, "Trigger", false, |ui| {
            widgets::property_row(ui, "Name", |ui| {
                ui.add_sized(
                    [ui.available_width().max(80.0), 24.0],
                    egui::TextEdit::singleline(&mut self.trigger_name_edit),
                );
            });
            widgets::property_row(ui, "Once", |ui| {
                ui.checkbox(&mut self.trigger_once_edit, "");
            });
            widgets::property_row(ui, "Action", |ui| {
                if widgets::toolbar_button(ui, "Apply", "Apply trigger", true, false).clicked() {
                    let mut trigger = TriggerComponent::new(self.trigger_name_edit.trim());
                    if self.trigger_once_edit {
                        trigger = trigger.once();
                    }
                    match self
                        .app
                        .execute_command_with_history(EditorCommand::attach_trigger(
                            entity, trigger,
                        )) {
                        Ok(_) => self.set_success("Trigger updated"),
                        Err(error) => self.set_error(format!("{error}")),
                    }
                }
            });
            widgets::property_row(ui, "Remove", |ui| {
                if widgets::toolbar_button(ui, "Remove Component", "Remove trigger", true, false)
                    .clicked()
                {
                    self.remove_component(entity, EditorComponentKind::Trigger, "Trigger removed");
                }
            });
        });
    }
}
