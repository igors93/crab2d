use super::*;

impl Crab2DEditorUi {
    pub(super) fn show_selected_node_header(
        &mut self,
        ui: &mut egui::Ui,
        entity: EntityId,
        name: &str,
    ) {
        let theme = theme();
        egui::Frame::new()
            .fill(theme.colors.panel_header_bg)
            .stroke(egui::Stroke::new(1.0, theme.colors.border))
            .corner_radius(theme.radius.sm)
            .inner_margin(egui::Margin::same(theme.spacing.section))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Node icon placeholder
                    let icon_rect =
                        egui::Rect::from_min_size(ui.cursor().min, egui::vec2(36.0, 36.0));
                    let painter = ui.painter_at(icon_rect);
                    painter.rect_filled(icon_rect, theme.radius.sm, theme.colors.control_active);
                    painter.text(
                        icon_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "N",
                        egui::FontId::proportional(15.0),
                        theme.colors.accent,
                    );
                    ui.allocate_space(egui::vec2(36.0, 36.0));
                    ui.add_space(8.0);

                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(truncate_text(name, 28))
                                .strong()
                                .size(15.0)
                                .color(theme.colors.text),
                        );
                        ui.label(
                            egui::RichText::new(self.node_type(entity))
                                .size(11.0)
                                .color(theme.colors.text_muted),
                        );
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        if widgets::toolbar_button(ui, "Del", "Delete node", true, false).clicked()
                        {
                            match self
                                .app
                                .execute_command_with_history(EditorCommand::delete_node(entity))
                            {
                                Ok(_) => {
                                    self.selected = None;
                                    self.select_default_node();
                                    self.set_success("Node deleted");
                                }
                                Err(error) => self.set_error(format!("{error}")),
                            }
                        }
                        if widgets::toolbar_button(ui, "Dup", "Duplicate node", true, false)
                            .clicked()
                        {
                            match self
                                .app
                                .execute_command_with_history(EditorCommand::duplicate_node(entity))
                            {
                                Ok(EditorCommandResult::CreatedNode(id)) => {
                                    self.select_node(id);
                                    self.set_success("Node duplicated");
                                }
                                Ok(EditorCommandResult::None) => {}
                                Err(error) => self.set_error(format!("{error}")),
                            }
                        }
                    });
                });
            });
        ui.add_space(theme.spacing.xs);
    }

    pub(super) fn show_add_component_panel(&mut self, ui: &mut egui::Ui, entity: EntityId) {
        widgets::inspector_section(ui, "Add Component", true, |ui| {
            let mut shown = false;
            ui.horizontal_wrapped(|ui| {
                if self.app.node_tag(entity).is_none() {
                    shown = true;
                    if widgets::toolbar_button(ui, "Tag", "Attach tag", true, false).clicked() {
                        self.attach_default_tag(entity);
                    }
                }
                if self.app.node_sprite(entity).is_none() {
                    shown = true;
                    if widgets::toolbar_button(ui, "Sprite", "Attach sprite", true, false).clicked()
                    {
                        self.attach_default_sprite(entity);
                    }
                }
                if self.app.node_tilemap(entity).is_none() {
                    shown = true;
                    if widgets::toolbar_button(ui, "Tilemap", "Attach tilemap", true, false)
                        .clicked()
                    {
                        self.attach_default_tilemap(entity);
                    }
                }
                if self.app.node_camera(entity).is_none() {
                    shown = true;
                    if widgets::toolbar_button(ui, "Camera2D", "Attach camera", true, false)
                        .clicked()
                    {
                        self.attach_default_camera(entity);
                    }
                }
                if self.app.node_velocity(entity).is_none() {
                    shown = true;
                    if widgets::toolbar_button(ui, "Velocity", "Attach velocity", true, false)
                        .clicked()
                    {
                        self.attach_default_velocity(entity);
                    }
                }
                if self.app.node_collider(entity).is_none() {
                    shown = true;
                    if widgets::toolbar_button(ui, "Collider2D", "Attach collider", true, false)
                        .clicked()
                    {
                        self.attach_default_collider(entity);
                    }
                }
                if self.app.node_player_controller(entity).is_none() {
                    shown = true;
                    if widgets::toolbar_button(
                        ui,
                        "PlayerController",
                        "Attach player controller",
                        true,
                        false,
                    )
                    .clicked()
                    {
                        self.attach_default_player_controller(entity);
                    }
                }
                if self.app.node_camera_follow(entity).is_none() {
                    shown = true;
                    if widgets::toolbar_button(
                        ui,
                        "CameraFollow",
                        "Attach camera follow",
                        true,
                        false,
                    )
                    .clicked()
                    {
                        self.attach_default_camera_follow(entity);
                    }
                }
                if self.app.node_trigger(entity).is_none() {
                    shown = true;
                    if widgets::toolbar_button(ui, "Trigger", "Attach trigger", true, false)
                        .clicked()
                    {
                        self.attach_default_trigger(entity);
                    }
                }
            });

            if !shown {
                ui.label(
                    egui::RichText::new("All supported components are attached")
                        .color(theme().colors.text_muted),
                );
            }
        });
    }

    fn attach_default_tag(&mut self, entity: EntityId) {
        let tag = match self.tag_edit.trim() {
            "" => "tag",
            value => value,
        }
        .to_owned();
        match self
            .app
            .execute_command_with_history(EditorCommand::attach_tag(entity, tag))
        {
            Ok(_) => {
                self.sync_selected_buffers();
                self.set_success("Tag added");
            }
            Err(error) => self.set_error(format!("{error}")),
        }
    }

    fn attach_default_sprite(&mut self, entity: EntityId) {
        let sprite_path = match self.sprite_edit.trim() {
            "" => "sprites/player.png",
            value => value,
        }
        .to_owned();
        self.apply_sprite(entity, sprite_path);
    }

    fn attach_default_tilemap(&mut self, entity: EntityId) {
        match default_tilemap() {
            Ok(tilemap) => match self
                .app
                .execute_command_with_history(EditorCommand::attach_tilemap(entity, tilemap))
            {
                Ok(_) => {
                    self.sync_selected_buffers();
                    self.set_success("Tilemap added");
                }
                Err(error) => self.set_error(format!("{error}")),
            },
            Err(error) => self.set_error(format!("{error}")),
        }
    }

    fn attach_default_camera(&mut self, entity: EntityId) {
        match self
            .app
            .execute_command_with_history(EditorCommand::attach_camera(
                entity,
                Camera2DComponent::default(),
            )) {
            Ok(_) => {
                self.sync_selected_buffers();
                self.set_success("Camera added");
            }
            Err(error) => self.set_error(format!("{error}")),
        }
    }

    fn attach_default_velocity(&mut self, entity: EntityId) {
        match self
            .app
            .execute_command_with_history(EditorCommand::attach_velocity(
                entity,
                Velocity2DComponent::ZERO,
            )) {
            Ok(_) => {
                self.sync_selected_buffers();
                self.set_success("Velocity added");
            }
            Err(error) => self.set_error(format!("{error}")),
        }
    }

    fn attach_default_collider(&mut self, entity: EntityId) {
        let collider = Collider2DComponent::rectangle(Vec2::new(24.0, 24.0));
        match self
            .app
            .execute_command_with_history(EditorCommand::attach_collider(entity, collider))
        {
            Ok(_) => {
                self.sync_selected_buffers();
                self.set_success("Collider added");
            }
            Err(error) => self.set_error(format!("{error}")),
        }
    }

    fn attach_default_player_controller(&mut self, entity: EntityId) {
        match self
            .app
            .execute_command_with_history(EditorCommand::attach_player_controller(
                entity,
                PlayerControllerComponent::default(),
            )) {
            Ok(_) => {
                self.sync_selected_buffers();
                self.set_success("Player controller added");
            }
            Err(error) => self.set_error(format!("{error}")),
        }
    }

    fn attach_default_camera_follow(&mut self, entity: EntityId) {
        let target = self.default_camera_follow_target(entity);
        match self
            .app
            .execute_command_with_history(EditorCommand::attach_camera_follow(
                entity,
                CameraFollowComponent::new(target),
            )) {
            Ok(_) => {
                self.sync_selected_buffers();
                self.set_success("Camera follow added");
            }
            Err(error) => self.set_error(format!("{error}")),
        }
    }

    fn attach_default_trigger(&mut self, entity: EntityId) {
        let trigger_name = match self.trigger_name_edit.trim() {
            "" => "trigger",
            value => value,
        };
        match self
            .app
            .execute_command_with_history(EditorCommand::attach_trigger(
                entity,
                TriggerComponent::new(trigger_name),
            )) {
            Ok(_) => {
                self.sync_selected_buffers();
                self.set_success("Trigger added");
            }
            Err(error) => self.set_error(format!("{error}")),
        }
    }

    fn default_camera_follow_target(&self, fallback: EntityId) -> EntityId {
        self.app
            .scene_nodes()
            .iter()
            .find(|node| self.app.node_player_controller(node.id).is_some())
            .map(|node| node.id)
            .unwrap_or(fallback)
    }
}
