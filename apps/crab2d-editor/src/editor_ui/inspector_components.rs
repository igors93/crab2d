use super::*;

impl Crab2DEditorUi {
    pub(super) fn show_tag_inspector(&mut self, ui: &mut egui::Ui, entity: EntityId) {
        if self.app.node_tag(entity).is_none() {
            widgets::inspector_section(ui, "Tag", false, |ui| {
                if widgets::toolbar_button(ui, "+ Add Tag", "Add tag component", true, false)
                    .clicked()
                {
                    let tag = self.tag_edit.trim().to_owned();
                    if !tag.is_empty() {
                        match self
                            .app
                            .execute_command_with_history(EditorCommand::attach_tag(entity, tag))
                        {
                            Ok(_) => self.set_success("Tag added"),
                            Err(error) => self.set_error(format!("{error}")),
                        }
                    }
                }
            });
            return;
        }

        widgets::inspector_section(ui, "Tag", true, |ui| {
            widgets::property_row(ui, "Value", |ui| {
                ui.add_sized(
                    [ui.available_width().max(80.0), 24.0],
                    egui::TextEdit::singleline(&mut self.tag_edit),
                );
            });
            ui.horizontal(|ui| {
                ui.add_space(theme().sizing.property_label_width + theme().spacing.sm);
                if widgets::toolbar_button(ui, "Apply", "Apply tag", true, false).clicked() {
                    let tag = self.tag_edit.trim().to_owned();
                    if !tag.is_empty() {
                        match self
                            .app
                            .execute_command_with_history(EditorCommand::attach_tag(entity, tag))
                        {
                            Ok(_) => self.set_success("Tag applied"),
                            Err(error) => self.set_error(format!("{error}")),
                        }
                    }
                }
            });

            if let Some(tag) = self.app.node_tag(entity) {
                widgets::property_row(ui, "Current", |ui| {
                    ui.monospace(&tag.tag);
                });
            }
            widgets::property_row(ui, "Remove", |ui| {
                if widgets::toolbar_button(ui, "Remove Component", "Remove tag", true, false)
                    .clicked()
                {
                    self.remove_component(entity, EditorComponentKind::Tag, "Tag removed");
                }
            });
        });
    }

    pub(super) fn show_sprite_inspector(&mut self, ui: &mut egui::Ui, entity: EntityId) {
        if self.app.node_sprite(entity).is_none() {
            widgets::inspector_section(ui, "Sprite", false, |ui| {
                if widgets::toolbar_button(ui, "+ Add Sprite", "Add sprite component", true, false)
                    .clicked()
                {
                    let sprite_path = self.sprite_edit.trim().to_owned();
                    if !sprite_path.is_empty() {
                        self.apply_sprite(entity, sprite_path);
                    }
                }
            });
            return;
        }

        widgets::inspector_section(ui, "Sprite", true, |ui| {
            widgets::property_row(ui, "Path", |ui| {
                ui.add_sized(
                    [ui.available_width().max(80.0), 24.0],
                    egui::TextEdit::singleline(&mut self.sprite_edit),
                );
            });
            ui.horizontal(|ui| {
                ui.add_space(theme().sizing.property_label_width + theme().spacing.sm);
                if widgets::toolbar_button(ui, "Apply", "Apply sprite path", true, false).clicked()
                {
                    let sprite_path = self.sprite_edit.trim().to_owned();
                    if !sprite_path.is_empty() {
                        self.apply_sprite(entity, sprite_path);
                    }
                }
            });

            if let Some(sprite) = self.app.node_sprite(entity) {
                widgets::property_row(ui, "Visible", |ui| {
                    ui.label(if sprite.visible { "Yes" } else { "No" });
                });
                widgets::property_row(ui, "Z Index", |ui| {
                    ui.label(sprite.z_index.to_string());
                });
            }
            widgets::property_row(ui, "Remove", |ui| {
                if widgets::toolbar_button(ui, "Remove Component", "Remove sprite", true, false)
                    .clicked()
                {
                    self.remove_component(entity, EditorComponentKind::Sprite, "Sprite removed");
                }
            });
        });
    }

    pub(super) fn show_tilemap_inspector(&mut self, ui: &mut egui::Ui, entity: EntityId) {
        if self.app.node_tilemap(entity).is_none() {
            widgets::inspector_section(ui, "Tilemap", false, |ui| {
                if widgets::toolbar_button(
                    ui,
                    "+ Add Tilemap",
                    "Add tilemap component",
                    true,
                    false,
                )
                .clicked()
                {
                    match default_tilemap() {
                        Ok(tilemap) => {
                            match self.app.execute_command_with_history(
                                EditorCommand::attach_tilemap(entity, tilemap),
                            ) {
                                Ok(_) => self.set_success("Tilemap added"),
                                Err(error) => self.set_error(format!("{error}")),
                            }
                        }
                        Err(error) => self.set_error(format!("{error}")),
                    }
                }
            });
            return;
        }

        if let Some(tilemap) = self.app.node_tilemap(entity) {
            let map_width = tilemap.map_size.width;
            let map_height = tilemap.map_size.height;
            let tile_width = tilemap.tile_size.width;
            let tile_height = tilemap.tile_size.height;
            let layer_count = tilemap.layers.len();

            widgets::inspector_section(ui, "Tilemap", true, |ui| {
                widgets::property_row(ui, "Map Size", |ui| {
                    ui.monospace(format!("{map_width} x {map_height}"));
                });
                widgets::property_row(ui, "Tile Size", |ui| {
                    ui.monospace(format!("{tile_width} x {tile_height}"));
                });
                widgets::property_row(ui, "Layers", |ui| {
                    ui.label(layer_count.to_string());
                });
                widgets::property_row(ui, "Solid Tiles", |ui| {
                    ui.add_sized(
                        [ui.available_width().max(80.0), 24.0],
                        egui::TextEdit::singleline(&mut self.tile_collision_edit)
                            .hint_text("3, 4, 7"),
                    );
                });
                widgets::property_row(ui, "Collision", |ui| {
                    if widgets::toolbar_button(ui, "Apply", "Apply solid tile indices", true, false)
                        .clicked()
                    {
                        match parse_solid_tiles(&self.tile_collision_edit) {
                            Ok(solid_tiles) => {
                                match self.app.execute_command_with_history(
                                    EditorCommand::set_tile_collision(entity, solid_tiles),
                                ) {
                                    Ok(_) => self.set_success("Tile collision updated"),
                                    Err(error) => self.set_error(format!("{error}")),
                                }
                            }
                            Err(error) => self.set_error(error),
                        }
                    }
                });
                widgets::property_row(ui, "Tool", |ui| {
                    if widgets::toolbar_button(ui, "Use Brush", "Activate tile brush", true, false)
                        .clicked()
                    {
                        self.active_tool = EditorTool::TileBrush;
                        self.bottom_tab = BottomDockTab::TilePalette;
                        self.set_status("Tile Brush active");
                    }
                });
                widgets::property_row(ui, "Remove", |ui| {
                    if widgets::toolbar_button(
                        ui,
                        "Remove Component",
                        "Remove tilemap",
                        true,
                        false,
                    )
                    .clicked()
                    {
                        self.remove_component(
                            entity,
                            EditorComponentKind::Tilemap,
                            "Tilemap removed",
                        );
                    }
                });
            });
        }
    }

    pub(super) fn show_camera_inspector(&mut self, ui: &mut egui::Ui, entity: EntityId) {
        if self.app.node_camera(entity).is_none() {
            widgets::inspector_section(ui, "Camera2D", false, |ui| {
                if widgets::toolbar_button(ui, "+ Add Camera", "Add camera component", true, false)
                    .clicked()
                {
                    match self
                        .app
                        .execute_command_with_history(EditorCommand::attach_camera(
                            entity,
                            Camera2DComponent::default(),
                        )) {
                        Ok(_) => self.set_success("Camera added"),
                        Err(error) => self.set_error(format!("{error}")),
                    }
                }
            });
            return;
        }

        if let Some(camera) = self.app.node_camera(entity) {
            let zoom = camera.zoom;
            widgets::inspector_section(ui, "Camera2D", false, |ui| {
                widgets::property_row(ui, "Zoom", |ui| {
                    ui.label(format!("{zoom:.2}"));
                });
                widgets::property_row(ui, "Frame", |ui| {
                    ui.label("640 x 360");
                });
                widgets::property_row(ui, "Remove", |ui| {
                    if widgets::toolbar_button(ui, "Remove Component", "Remove camera", true, false)
                        .clicked()
                    {
                        self.remove_component(
                            entity,
                            EditorComponentKind::Camera,
                            "Camera removed",
                        );
                    }
                });
            });
        }
    }
}
