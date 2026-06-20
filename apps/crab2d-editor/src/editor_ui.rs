use crab2d_editor::{EditorApp, EditorCommand, EditorCommandResult, EntityId};
use eframe::egui;

pub struct Crab2DEditorUi {
    app: EditorApp,
    selected: Option<EntityId>,
    // Buffered name for the inspector text field.
    // Synced from the engine whenever the field is not focused,
    // so undo/redo changes are reflected immediately.
    name_edit: String,
    status: String,
}

impl Crab2DEditorUi {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = EditorApp::new("Crab2D Editor");
        app.open_empty_project("Untitled Project");
        Self {
            app,
            selected: None,
            name_edit: String::new(),
            status: String::new(),
        }
    }

    fn select_node(&mut self, id: EntityId) {
        if self.selected != Some(id) {
            self.selected = Some(id);
            if let Some(node) = self.app.find_node(id) {
                self.name_edit = node.name.clone();
            }
        }
    }
}

impl eframe::App for Crab2DEditorUi {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Global keyboard shortcuts
        let (want_undo, want_redo) = ctx.input(|i| {
            let undo = i.key_pressed(egui::Key::Z) && i.modifiers.ctrl && !i.modifiers.shift;
            let redo = (i.key_pressed(egui::Key::Z) && i.modifiers.ctrl && i.modifiers.shift)
                || (i.key_pressed(egui::Key::Y) && i.modifiers.ctrl);
            (undo, redo)
        });
        if want_undo {
            if let Err(e) = self.app.undo() {
                self.status = format!("{e}");
            } else {
                self.status.clear();
            }
        }
        if want_redo {
            if let Err(e) = self.app.redo() {
                self.status = format!("{e}");
            } else {
                self.status.clear();
            }
        }

        self.show_menu_bar(ui);
        self.show_status_bar(ui);
        self.show_scene_panel(ui);
        self.show_inspector_panel(ui);
        self.show_viewport(ui);
    }
}

impl Crab2DEditorUi {
    fn show_menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::top("menu_bar").show_inside(ui, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Project").clicked() {
                        self.app.open_empty_project("Untitled Project");
                        self.selected = None;
                        self.name_edit.clear();
                        self.status = "New project created.".to_owned();
                        ui.close_menu();
                    }
                    if ui.button("Save").clicked() {
                        match self.app.save_current_project_to_default_file() {
                            Ok(()) => self.status = "Project saved.".to_owned(),
                            Err(e) => self.status = format!("Save failed: {e}"),
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    let can_undo = self.app.can_undo();
                    let can_redo = self.app.can_redo();
                    if ui
                        .add_enabled(can_undo, egui::Button::new("Undo  Ctrl+Z"))
                        .clicked()
                    {
                        if let Err(e) = self.app.undo() {
                            self.status = format!("{e}");
                        } else {
                            self.status.clear();
                        }
                        ui.close_menu();
                    }
                    if ui
                        .add_enabled(can_redo, egui::Button::new("Redo  Ctrl+Y"))
                        .clicked()
                    {
                        if let Err(e) = self.app.redo() {
                            self.status = format!("{e}");
                        } else {
                            self.status.clear();
                        }
                        ui.close_menu();
                    }
                });

                ui.separator();

                if ui
                    .add_enabled(self.app.can_undo(), egui::Button::new("Undo"))
                    .on_hover_text("Ctrl+Z")
                    .clicked()
                {
                    if let Err(e) = self.app.undo() {
                        self.status = format!("{e}");
                    } else {
                        self.status.clear();
                    }
                }
                if ui
                    .add_enabled(self.app.can_redo(), egui::Button::new("Redo"))
                    .on_hover_text("Ctrl+Y")
                    .clicked()
                {
                    if let Err(e) = self.app.redo() {
                        self.status = format!("{e}");
                    } else {
                        self.status.clear();
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.weak(self.app.project_name());
                });
            });
        });
    }

    fn show_status_bar(&self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::bottom("status_bar").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                let msg = if self.status.is_empty() {
                    "Ready"
                } else {
                    &self.status
                };
                ui.weak(msg);
            });
        });
    }

    fn show_scene_panel(&mut self, ui: &mut egui::Ui) {
        egui::SidePanel::left("scene_panel")
            .resizable(true)
            .default_size(200.0)
            .min_size(140.0)
            .show_inside(ui, |ui| {
                ui.heading("Scene");
                ui.separator();

                // Collect ids first to avoid holding a borrow while mutating self
                let ids: Vec<EntityId> = self.app.scene_nodes().iter().map(|n| n.id).collect();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for id in ids {
                        let label = self.node_label(id);
                        let is_selected = self.selected == Some(id);
                        if ui.selectable_label(is_selected, label).clicked() {
                            self.select_node(id);
                        }
                    }
                });

                ui.separator();
                if ui.button("+ Add Node").clicked() {
                    match self
                        .app
                        .execute_command_with_history(EditorCommand::create_node("Node"))
                    {
                        Ok(EditorCommandResult::CreatedNode(id)) => {
                            self.select_node(id);
                            self.status.clear();
                        }
                        Err(e) => self.status = format!("{e}"),
                        Ok(EditorCommandResult::None) => {}
                    }
                }
            });
    }

    // Returns a display label for a node that hints at its attached components.
    fn node_label(&self, id: EntityId) -> String {
        let name = self
            .app
            .find_node(id)
            .map(|n| n.name.as_str())
            .unwrap_or("?");

        if self.app.node_camera(id).is_some() {
            format!("{name}  [cam]")
        } else if self.app.node_sprite(id).is_some() {
            format!("{name}  [spr]")
        } else if self.app.node_tag(id).is_some() {
            format!("{name}  [tag]")
        } else {
            name.to_owned()
        }
    }

    fn show_inspector_panel(&mut self, ui: &mut egui::Ui) {
        egui::SidePanel::right("inspector_panel")
            .resizable(true)
            .default_size(240.0)
            .min_size(180.0)
            .show_inside(ui, |ui| {
                ui.heading("Inspector");
                ui.separator();

                let Some(entity) = self.selected else {
                    ui.weak("No node selected.");
                    return;
                };

                // Clone node data up front to release the immutable borrow
                // before any mutable calls (execute_command_with_history, etc.)
                let Some((current_name, pos_x, pos_y, rot_deg)) =
                    self.app.find_node(entity).map(|n| {
                        (
                            n.name.clone(),
                            n.transform.position.x,
                            n.transform.position.y,
                            n.transform.rotation_radians.to_degrees(),
                        )
                    })
                else {
                    self.selected = None;
                    return;
                };

                let tag_value = self.app.node_tag(entity).map(|t| t.tag.clone());
                let sprite_info = self
                    .app
                    .node_sprite(entity)
                    .map(|s| (s.sprite_path.clone(), s.visible));
                let has_camera = self.app.node_camera(entity).is_some();

                // ── Name ──────────────────────────────────────────────────────
                ui.label("Name");
                let name_response = ui.text_edit_singleline(&mut self.name_edit);

                if name_response.lost_focus() {
                    let trimmed = self.name_edit.trim().to_owned();
                    if !trimmed.is_empty() && trimmed != current_name {
                        if let Err(e) =
                            self.app
                                .execute_command_with_history(EditorCommand::rename_node(
                                    entity, &trimmed,
                                ))
                        {
                            self.status = format!("{e}");
                        } else {
                            self.status.clear();
                        }
                    }
                    // Resync buffer after committing (also picks up rejection)
                    if let Some(n) = self.app.find_node(entity) {
                        self.name_edit = n.name.clone();
                    }
                } else if !name_response.has_focus() {
                    // Not being edited — mirror any external changes (undo/redo)
                    self.name_edit = current_name.clone();
                }

                // ── Transform ─────────────────────────────────────────────────
                ui.add_space(8.0);
                egui::CollapsingHeader::new("Transform")
                    .default_open(true)
                    .show(ui, |ui| {
                        egui::Grid::new("transform_grid")
                            .num_columns(2)
                            .spacing([8.0, 4.0])
                            .show(ui, |ui| {
                                ui.weak("X");
                                ui.label(format!("{pos_x:.2}"));
                                ui.end_row();
                                ui.weak("Y");
                                ui.label(format!("{pos_y:.2}"));
                                ui.end_row();
                                ui.weak("Rotation");
                                ui.label(format!("{rot_deg:.1}°"));
                                ui.end_row();
                            });
                    });

                // ── Camera2D ──────────────────────────────────────────────────
                if has_camera {
                    ui.add_space(8.0);
                    egui::CollapsingHeader::new("Camera2D")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.weak("Active camera");
                        });
                }

                // ── Tag ───────────────────────────────────────────────────────
                if let Some(tag) = tag_value {
                    ui.add_space(8.0);
                    egui::CollapsingHeader::new("Tag")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.monospace(&tag);
                        });
                }

                // ── Sprite ────────────────────────────────────────────────────
                if let Some((path, visible)) = sprite_info {
                    ui.add_space(8.0);
                    egui::CollapsingHeader::new("Sprite")
                        .default_open(true)
                        .show(ui, |ui| {
                            egui::Grid::new("sprite_grid")
                                .num_columns(2)
                                .spacing([8.0, 4.0])
                                .show(ui, |ui| {
                                    ui.weak("Path");
                                    ui.monospace(&path);
                                    ui.end_row();
                                    ui.weak("Visible");
                                    ui.label(if visible { "Yes" } else { "No" });
                                    ui.end_row();
                                });
                        });
                }
            });
    }

    fn show_viewport(&self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let rect = ui.available_rect_before_wrap();
            ui.painter()
                .rect_filled(rect, 0.0, egui::Color32::from_rgb(28, 28, 32));
            ui.centered_and_justified(|ui| {
                ui.weak("Viewport\n(coming soon)");
            });
        });
    }
}
