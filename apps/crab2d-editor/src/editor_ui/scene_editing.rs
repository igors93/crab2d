use super::*;

impl Crab2DEditorUi {
    pub(super) fn begin_asset_drag(&mut self, asset_path: String, display_name: String) {
        self.selected_asset_path = Some(asset_path.clone());
        self.asset_drag = Some(AssetPlacementDrag {
            asset_path,
            display_name,
        });
        self.set_status("Drag asset into the scene");
    }

    pub(super) fn finish_asset_drag_at(&mut self, world_position: Vec2) {
        let Some(drag) = self.asset_drag.take() else {
            return;
        };
        self.create_asset_node_at(drag.asset_path, drag.display_name.as_str(), world_position);
    }

    pub(super) fn cancel_asset_drag(&mut self) {
        if self.asset_drag.take().is_some() {
            self.set_status("Asset placement cancelled");
        }
    }

    pub(super) fn create_asset_node_at(
        &mut self,
        asset_path: String,
        display_name: &str,
        world_position: Vec2,
    ) {
        let node_name = node_name_from_asset(display_name, asset_path.as_str());
        let transform = Transform2D::from_position(world_position);
        match self
            .app
            .execute_command_with_history(EditorCommand::create_from_asset_at(
                node_name.clone(),
                asset_path.clone(),
                transform,
            )) {
            Ok(EditorCommandResult::CreatedNode(entity)) => {
                self.select_node(entity);
                self.sprite_edit = asset_path;
                self.set_success(format!("{node_name} placed"));
            }
            Ok(EditorCommandResult::None) => {}
            Err(error) => self.set_error(format!("{error}")),
        }
    }

    pub(super) fn show_viewport_context_menu(&mut self, ui: &mut egui::Ui) {
        let theme = theme();
        if let Some(entity) = self.selected {
            let node_name = self
                .app
                .find_node(entity)
                .map(|node| node.name.clone())
                .unwrap_or_else(|| "Selected".to_owned());
            ui.label(
                egui::RichText::new(format!("Rules for {}", truncate_text(&node_name, 24)))
                    .strong()
                    .color(theme.colors.text),
            );
            ui.add_space(theme.spacing.xs);
            self.show_preset_buttons(ui, |editor, preset| {
                editor.apply_preset_to_entity(entity, preset);
            });
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Duplicate").clicked() {
                    self.duplicate_entity(entity);
                    ui.close();
                }
                if ui.button("Delete").clicked() {
                    self.delete_entity(entity);
                    ui.close();
                }
            });
            ui.separator();
        }

        if let Some(world_position) = self.viewport_context_world {
            ui.label(
                egui::RichText::new("Create here")
                    .strong()
                    .color(theme.colors.text),
            );
            ui.add_space(theme.spacing.xs);

            if let Some(asset_path) = self.selected_asset_path.clone() {
                let label = format!(
                    "Place {}",
                    truncate_text(
                        std::path::Path::new(asset_path.as_str())
                            .file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or(asset_path.as_str()),
                        22,
                    )
                );
                if ui.button(label).clicked() {
                    let display_name = asset_path
                        .rsplit('/')
                        .next()
                        .unwrap_or(asset_path.as_str())
                        .to_owned();
                    self.create_asset_node_at(asset_path, display_name.as_str(), world_position);
                    ui.close();
                }
                ui.separator();
            }

            self.show_preset_buttons(ui, |editor, preset| {
                editor.create_preset_node_at(preset, world_position);
            });
        }
    }

    fn show_preset_buttons(
        &mut self,
        ui: &mut egui::Ui,
        mut action: impl FnMut(&mut Self, GameplayPreset),
    ) {
        for preset in [
            GameplayPreset::TopDownPlayer,
            GameplayPreset::StaticWall,
            GameplayPreset::Collectible,
            GameplayPreset::Door,
            GameplayPreset::TriggerArea,
            GameplayPreset::DamageZone,
            GameplayPreset::Checkpoint,
            GameplayPreset::CameraFollow,
            GameplayPreset::WorldSign,
            GameplayPreset::HudLabel,
        ] {
            if ui.button(preset.label()).clicked() {
                action(self, preset);
                ui.close();
            }
        }
    }

    fn apply_preset_to_entity(&mut self, entity: EntityId, preset: GameplayPreset) {
        match self
            .app
            .execute_command_with_history(EditorCommand::apply_gameplay_preset(entity, preset))
        {
            Ok(_) => {
                self.select_node(entity);
                self.sync_selected_buffers();
                self.set_success(format!("{} rule applied", preset.label()));
            }
            Err(error) => self.set_error(format!("{error}")),
        }
    }

    fn create_preset_node_at(&mut self, preset: GameplayPreset, world_position: Vec2) {
        match self.app.create_preset_node(preset) {
            Ok(entity) => {
                if let Some(before) = self.app.node_transform(entity) {
                    let mut transform = before;
                    transform.position = world_position;
                    match self
                        .app
                        .execute_command(EditorCommand::move_node(entity, transform))
                    {
                        Ok(_) => self.app.record_move_node(entity, before, transform),
                        Err(error) => {
                            self.set_error(format!("{error}"));
                            return;
                        }
                    }
                }
                self.select_node(entity);
                self.set_success(format!("{} created", preset.label()));
            }
            Err(error) => self.set_error(format!("{error}")),
        }
    }

    fn duplicate_entity(&mut self, entity: EntityId) {
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

    fn delete_entity(&mut self, entity: EntityId) {
        match self
            .app
            .execute_command_with_history(EditorCommand::delete_node(entity))
        {
            Ok(_) => {
                self.select_default_node();
                self.set_success("Node deleted");
            }
            Err(error) => self.set_error(format!("{error}")),
        }
    }
}

fn node_name_from_asset(display_name: &str, asset_path: &str) -> String {
    let source = if display_name.trim().is_empty() {
        asset_path
    } else {
        display_name
    };
    let stem = std::path::Path::new(source)
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("Sprite");
    let mut name = String::new();
    let mut uppercase_next = true;
    for character in stem.chars() {
        if character == '_' || character == '-' || character == ' ' {
            uppercase_next = true;
            continue;
        }
        if uppercase_next {
            name.extend(character.to_uppercase());
            uppercase_next = false;
        } else {
            name.push(character);
        }
    }
    if name.is_empty() {
        "Sprite".to_owned()
    } else {
        name
    }
}
