use crab2d_editor::{EditorApp, EditorCommand, EditorCommandResult, EntityId, Vec2};
use eframe::egui;

pub struct Crab2DEditorUi {
    app: EditorApp,
    selected: Option<EntityId>,
    name_edit: String,
    filter_edit: String,
    tag_edit: String,
    sprite_edit: String,
    status: String,
    output: Vec<String>,
}

impl Crab2DEditorUi {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_style(&cc.egui_ctx);

        let mut app = EditorApp::new("Crab2D Editor");
        app.open_empty_project("Untitled Project");
        let selected = Self::default_selected_node(&app);
        let name_edit = selected
            .and_then(|id| app.find_node(id))
            .map(|node| node.name.clone())
            .unwrap_or_default();
        let tag_edit = selected
            .and_then(|id| app.node_tag(id))
            .map(|tag| tag.tag.clone())
            .unwrap_or_else(|| "player".to_owned());
        let sprite_edit = selected
            .and_then(|id| app.node_sprite(id))
            .map(|sprite| sprite.sprite_path.clone())
            .unwrap_or_else(|| "sprites/player.png".to_owned());

        Self {
            app,
            selected,
            name_edit,
            filter_edit: String::new(),
            tag_edit,
            sprite_edit,
            status: "Ready".to_owned(),
            output: vec![
                "[INFO] Crab2D editor started".to_owned(),
                "[INFO] Untitled Project loaded".to_owned(),
            ],
        }
    }

    fn default_selected_node(app: &EditorApp) -> Option<EntityId> {
        app.scene_nodes()
            .iter()
            .find(|node| node.name == "Player")
            .or_else(|| app.scene_nodes().first())
            .map(|node| node.id)
    }

    fn select_default_node(&mut self) {
        self.selected = Self::default_selected_node(&self.app);
        self.sync_selected_buffers();
    }

    fn select_node(&mut self, id: EntityId) {
        if self.selected == Some(id) {
            return;
        }

        self.selected = Some(id);
        self.sync_selected_buffers();
    }

    fn sync_selected_buffers(&mut self) {
        let Some(id) = self.selected else {
            self.name_edit.clear();
            self.tag_edit = "player".to_owned();
            self.sprite_edit = "sprites/player.png".to_owned();
            return;
        };

        if let Some(node) = self.app.find_node(id) {
            self.name_edit = node.name.clone();
        }
        self.tag_edit = self
            .app
            .node_tag(id)
            .map(|tag| tag.tag.clone())
            .unwrap_or_else(|| "player".to_owned());
        self.sprite_edit = self
            .app
            .node_sprite(id)
            .map(|sprite| sprite.sprite_path.clone())
            .unwrap_or_else(|| "sprites/player.png".to_owned());
    }

    fn set_status(&mut self, message: impl Into<String>) {
        let message = message.into();
        self.status = message.clone();
        if !message.is_empty() {
            self.output.push(format!("[INFO] {message}"));
        }
    }

    fn set_error(&mut self, message: impl Into<String>) {
        let message = message.into();
        self.status = message.clone();
        self.output.push(format!("[ERROR] {message}"));
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let (want_undo, want_redo, want_save) = ctx.input(|input| {
            let undo =
                input.key_pressed(egui::Key::Z) && input.modifiers.ctrl && !input.modifiers.shift;
            let redo =
                (input.key_pressed(egui::Key::Z) && input.modifiers.ctrl && input.modifiers.shift)
                    || (input.key_pressed(egui::Key::Y) && input.modifiers.ctrl);
            let save = input.key_pressed(egui::Key::S) && input.modifiers.ctrl;
            (undo, redo, save)
        });

        if want_undo {
            self.undo();
        }
        if want_redo {
            self.redo();
        }
        if want_save {
            self.save_project();
        }
    }

    fn new_project(&mut self) {
        self.app.open_empty_project("Untitled Project");
        self.filter_edit.clear();
        self.select_default_node();
        self.set_status("New project created");
    }

    fn save_project(&mut self) {
        match self.app.save_current_project_to_default_file() {
            Ok(()) => self.set_status("Project saved to project.crab2d.json"),
            Err(error) => self.set_error(format!("Save failed: {error}")),
        }
    }

    fn undo(&mut self) {
        match self.app.undo() {
            Ok(()) => {
                self.sync_selected_buffers();
                self.set_status("Undo");
            }
            Err(error) => self.set_error(format!("{error}")),
        }
    }

    fn redo(&mut self) {
        match self.app.redo() {
            Ok(()) => {
                self.sync_selected_buffers();
                self.set_status("Redo");
            }
            Err(error) => self.set_error(format!("{error}")),
        }
    }

    fn create_node(&mut self) {
        match self
            .app
            .execute_command_with_history(EditorCommand::create_node("Node"))
        {
            Ok(EditorCommandResult::CreatedNode(id)) => {
                self.select_node(id);
                self.set_status("Node created");
            }
            Ok(EditorCommandResult::None) => {}
            Err(error) => self.set_error(format!("{error}")),
        }
    }
}

impl eframe::App for Crab2DEditorUi {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.handle_shortcuts(&ctx);
        self.show_top_bar(ui, frame);
        self.show_scene_panel(ui);
        self.show_inspector(ui);
        self.show_bottom_dock(ui);
        self.show_viewport(ui);
    }
}

impl Crab2DEditorUi {
    fn show_top_bar(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("top_bar")
            .exact_size(58.0)
            .show_inside(root, |ui| {
                ui.add_space(6.0);
                ui.horizontal_centered(|ui| {
                    ui.add_space(8.0);
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Crab2D").strong().size(18.0));
                        ui.weak("v0.1.0 (Rust)");
                    });

                    ui.separator();

                    toolbar_button(ui, "New", "Ctrl+N", || self.new_project());
                    toolbar_button(ui, "Save", "Ctrl+S", || self.save_project());
                    toolbar_button(ui, "Undo", "Ctrl+Z", || self.undo());
                    toolbar_button(ui, "Redo", "Ctrl+Y", || self.redo());

                    ui.separator();

                    if ui
                        .add(egui::Button::new(
                            egui::RichText::new("Play").color(accent()),
                        ))
                        .on_hover_text("Preview mode")
                        .clicked()
                    {
                        self.app.preview_procedural_world();
                        self.set_status("Preview generated");
                    }
                    ui.add_enabled(false, egui::Button::new("Pause"));
                    ui.add_enabled(false, egui::Button::new("Stop"));

                    ui.separator();

                    let _ = ui.selectable_label(true, "Select");
                    ui.add_enabled(false, egui::Button::new("Tile Brush"));
                    ui.add_enabled(false, egui::Button::new("Collision"));
                    ui.add_enabled(false, egui::Button::new("Light"));
                    ui.add_enabled(false, egui::Button::new("View"));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new(self.app.project_name()).color(soft_text()));
                        ui.separator();
                        ui.add_enabled(false, egui::Button::new("Build / Export"));
                        ui.add_enabled(false, egui::Button::new("Plugin Market"));
                    });
                });
            });
    }

    fn show_scene_panel(&mut self, root: &mut egui::Ui) {
        egui::Panel::left("scene_panel")
            .resizable(true)
            .default_size(240.0)
            .min_size(180.0)
            .show_inside(root, |ui| {
                panel_header(ui, "SCENE");

                ui.horizontal(|ui| {
                    ui.add_sized(
                        [ui.available_width() - 38.0, 24.0],
                        egui::TextEdit::singleline(&mut self.filter_edit)
                            .hint_text("Filter nodes..."),
                    );
                    if ui.button("+").on_hover_text("Add node").clicked() {
                        self.create_node();
                    }
                });

                ui.add_space(8.0);

                let filter = self.filter_edit.to_lowercase();
                let ids: Vec<EntityId> = self
                    .app
                    .scene_nodes()
                    .iter()
                    .filter(|node| {
                        filter.is_empty() || node.name.to_lowercase().contains(filter.as_str())
                    })
                    .map(|node| node.id)
                    .collect();
                ui.push_id("scene_nodes_scroll", |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for id in ids {
                            let is_selected = self.selected == Some(id);
                            let response = ui.selectable_label(is_selected, self.node_label(id));
                            if response.clicked() {
                                self.select_node(id);
                            }
                        }
                    });
                });

                ui.separator();
                ui.horizontal(|ui| {
                    let _ = ui.selectable_label(true, "Scene");
                    let _ = ui.selectable_label(false, "Library");
                });

                ui.add_space(10.0);
                panel_header(ui, "PROPERTIES");
                ui.weak("World");
                ui.horizontal(|ui| {
                    ui.label("Size");
                    ui.monospace("2048 x 2048");
                });
                ui.horizontal(|ui| {
                    ui.label("Background");
                    ui.colored_label(accent(), "#2b2b2f");
                });
            });
    }

    fn show_inspector(&mut self, root: &mut egui::Ui) {
        egui::Panel::right("inspector_panel")
            .resizable(true)
            .default_size(330.0)
            .min_size(240.0)
            .show_inside(root, |ui| {
                panel_header(ui, "INSPECTOR");

                let Some(entity) = self.selected else {
                    ui.add_space(12.0);
                    ui.weak("No node selected.");
                    return;
                };

                let Some(node) = self.app.find_node(entity).cloned() else {
                    self.selected = None;
                    self.name_edit.clear();
                    return;
                };

                ui.horizontal(|ui| {
                    ui.weak("Node");
                    ui.monospace(format!("#{}", entity.raw()));
                });

                ui.add_space(8.0);
                inspector_section(ui, "Node", |ui| {
                    ui.label("Name");
                    let response = ui.text_edit_singleline(&mut self.name_edit);
                    let enter = ui.input(|input| input.key_pressed(egui::Key::Enter));
                    if response.lost_focus() || (response.has_focus() && enter) {
                        let name = self.name_edit.trim().to_owned();
                        if !name.is_empty() && name != node.name {
                            match self
                                .app
                                .execute_command_with_history(EditorCommand::rename_node(
                                    entity, &name,
                                )) {
                                Ok(_) => self.set_status("Node renamed"),
                                Err(error) => self.set_error(format!("{error}")),
                            }
                        }
                        self.sync_selected_buffers();
                    }
                });

                let mut transform = node.transform;
                inspector_section(ui, "Transform2D", |ui| {
                    let mut changed = false;
                    egui::Grid::new("transform_editor")
                        .num_columns(3)
                        .spacing([8.0, 6.0])
                        .show(ui, |ui| {
                            ui.weak("Position");
                            changed |= ui
                                .add(egui::DragValue::new(&mut transform.position.x).speed(1.0))
                                .changed();
                            changed |= ui
                                .add(egui::DragValue::new(&mut transform.position.y).speed(1.0))
                                .changed();
                            ui.end_row();

                            ui.weak("Scale");
                            changed |= ui
                                .add(egui::DragValue::new(&mut transform.scale.x).speed(0.05))
                                .changed();
                            changed |= ui
                                .add(egui::DragValue::new(&mut transform.scale.y).speed(0.05))
                                .changed();
                            ui.end_row();

                            ui.weak("Rotation");
                            let mut degrees = transform.rotation_radians.to_degrees();
                            changed |= ui
                                .add(egui::DragValue::new(&mut degrees).speed(1.0).suffix(" deg"))
                                .changed();
                            transform.rotation_radians = degrees.to_radians();
                            ui.end_row();
                        });

                    if changed {
                        if let Err(error) =
                            self.app
                                .execute_command_with_history(EditorCommand::move_node(
                                    entity, transform,
                                ))
                        {
                            self.set_error(format!("{error}"));
                        } else {
                            self.status = "Transform updated".to_owned();
                        }
                    }
                });

                inspector_section(ui, "Tag", |ui| {
                    ui.text_edit_singleline(&mut self.tag_edit);
                    if ui.button("Apply Tag").clicked() {
                        let tag = self.tag_edit.trim().to_owned();
                        if !tag.is_empty() {
                            match self
                                .app
                                .execute_command_with_history(EditorCommand::attach_tag(
                                    entity, tag,
                                )) {
                                Ok(_) => self.set_status("Tag applied"),
                                Err(error) => self.set_error(format!("{error}")),
                            }
                        }
                    }

                    if let Some(tag) = self.app.node_tag(entity) {
                        ui.weak(format!("Current: {}", tag.tag));
                    }
                });

                inspector_section(ui, "Sprite", |ui| {
                    ui.text_edit_singleline(&mut self.sprite_edit);
                    if ui.button("Apply Sprite").clicked() {
                        let sprite_path = self.sprite_edit.trim().to_owned();
                        if !sprite_path.is_empty() {
                            match self.app.execute_command_with_history(
                                EditorCommand::attach_sprite(entity, sprite_path),
                            ) {
                                Ok(_) => self.set_status("Sprite applied"),
                                Err(error) => self.set_error(format!("{error}")),
                            }
                        }
                    }

                    if let Some(sprite) = self.app.node_sprite(entity) {
                        ui.horizontal(|ui| {
                            ui.weak("Path");
                            ui.monospace(&sprite.sprite_path);
                        });
                        ui.horizontal(|ui| {
                            ui.weak("Visible");
                            ui.label(if sprite.visible { "Yes" } else { "No" });
                        });
                        ui.horizontal(|ui| {
                            ui.weak("Z Index");
                            ui.label(sprite.z_index.to_string());
                        });
                    }
                });

                if self.app.node_camera(entity).is_some() {
                    inspector_section(ui, "Camera2D", |ui| {
                        ui.weak("Active scene camera");
                    });
                }
            });
    }

    fn show_bottom_dock(&mut self, root: &mut egui::Ui) {
        egui::Panel::bottom("bottom_dock")
            .resizable(true)
            .default_size(170.0)
            .min_size(110.0)
            .show_inside(root, |ui| {
                ui.horizontal(|ui| {
                    let _ = ui.selectable_label(true, "ASSETS");
                    let _ = ui.selectable_label(false, "Sprites");
                    let _ = ui.selectable_label(false, "Audio");
                    let _ = ui.selectable_label(false, "Scripts");
                    let _ = ui.selectable_label(false, "Maps");
                });
                ui.separator();

                ui.columns(2, |columns| {
                    columns[0].push_id("asset_scroll", |ui| {
                        egui::ScrollArea::horizontal().show(ui, |ui| {
                            ui.horizontal(|ui| {
                                asset_tile(
                                    ui,
                                    "grass_tileset.png",
                                    egui::Color32::from_rgb(86, 132, 43),
                                );
                                asset_tile(
                                    ui,
                                    "stone_tileset.png",
                                    egui::Color32::from_rgb(108, 107, 96),
                                );
                                asset_tile(ui, "player.png", egui::Color32::from_rgb(204, 132, 52));
                                asset_tile(ui, "slime.png", egui::Color32::from_rgb(84, 180, 78));
                                asset_tile(ui, "props.png", egui::Color32::from_rgb(154, 104, 49));
                            });
                        });
                    });

                    columns[1].push_id("output_scroll", |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.monospace("OUTPUT");
                            for line in self.output.iter().rev().take(6) {
                                ui.colored_label(
                                    if line.contains("[ERROR]") {
                                        egui::Color32::from_rgb(245, 112, 112)
                                    } else {
                                        egui::Color32::from_rgb(132, 206, 117)
                                    },
                                    line,
                                );
                            }
                        });
                    });
                });
            });
    }

    fn show_viewport(&self, root: &mut egui::Ui) {
        egui::CentralPanel::default().show_inside(root, |ui| {
            let available = ui.available_size();
            let (rect, _) = ui.allocate_exact_size(available, egui::Sense::click_and_drag());
            let painter = ui.painter_at(rect);

            painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(16, 21, 24));
            draw_grid(
                &painter,
                rect,
                32.0,
                egui::Color32::from_rgba_unmultiplied(86, 132, 140, 58),
            );
            draw_grid(
                &painter,
                rect,
                128.0,
                egui::Color32::from_rgba_unmultiplied(84, 188, 190, 78),
            );

            let camera_rect = egui::Rect::from_center_size(rect.center(), rect.size() * 0.72);
            painter.rect_stroke(
                camera_rect,
                0.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(115, 89, 190)),
                egui::StrokeKind::Inside,
            );

            let world_to_screen = |position: Vec2| -> egui::Pos2 {
                rect.center() + egui::vec2(position.x, -position.y)
            };

            for (index, node) in self.app.scene_nodes().iter().enumerate() {
                let position = if node.transform.position == Vec2::ZERO {
                    Vec2::new(-180.0 + index as f32 * 72.0, 32.0 - index as f32 * 18.0)
                } else {
                    node.transform.position
                };
                let center = world_to_screen(position);
                let color = if Some(node.id) == self.selected {
                    accent()
                } else if self.app.node_camera(node.id).is_some() {
                    egui::Color32::from_rgb(120, 148, 255)
                } else if self.app.node_sprite(node.id).is_some() {
                    egui::Color32::from_rgb(102, 198, 91)
                } else {
                    egui::Color32::from_rgb(214, 166, 84)
                };

                let node_rect = egui::Rect::from_center_size(center, egui::vec2(34.0, 34.0));
                painter.rect_filled(node_rect, 4.0, color.gamma_multiply(0.65));
                painter.rect_stroke(
                    node_rect,
                    4.0,
                    egui::Stroke::new(1.5, color),
                    egui::StrokeKind::Inside,
                );
                painter.text(
                    center + egui::vec2(0.0, 26.0),
                    egui::Align2::CENTER_TOP,
                    &node.name,
                    egui::FontId::monospace(11.0),
                    soft_text(),
                );

                if Some(node.id) == self.selected {
                    painter.circle_stroke(center, 54.0, egui::Stroke::new(2.0, accent()));
                    painter.line_segment(
                        [center, center + egui::vec2(62.0, 0.0)],
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(237, 87, 87)),
                    );
                    painter.line_segment(
                        [center, center - egui::vec2(0.0, 62.0)],
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(83, 210, 100)),
                    );
                }
            }

            painter.text(
                rect.left_top() + egui::vec2(14.0, 12.0),
                egui::Align2::LEFT_TOP,
                "2D Viewport",
                egui::FontId::proportional(14.0),
                soft_text(),
            );
            painter.text(
                rect.right_bottom() - egui::vec2(14.0, 12.0),
                egui::Align2::RIGHT_BOTTOM,
                "Zoom: 100%   Tile: (34, 17)",
                egui::FontId::monospace(12.0),
                egui::Color32::from_rgb(130, 145, 150),
            );
        });
    }

    fn node_label(&self, id: EntityId) -> String {
        let name = self
            .app
            .find_node(id)
            .map(|node| node.name.as_str())
            .unwrap_or("?");

        if self.app.node_camera(id).is_some() {
            format!("camera  {name}")
        } else if self.app.node_sprite(id).is_some() {
            format!("sprite  {name}")
        } else if self.app.node_tag(id).is_some() {
            format!("tag     {name}")
        } else {
            format!("node    {name}")
        }
    }
}

fn configure_style(ctx: &egui::Context) {
    let mut style = (*ctx.global_style()).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.window_fill = egui::Color32::from_rgb(18, 24, 28);
    style.visuals.panel_fill = egui::Color32::from_rgb(18, 24, 28);
    style.visuals.faint_bg_color = egui::Color32::from_rgb(24, 31, 36);
    style.visuals.extreme_bg_color = egui::Color32::from_rgb(10, 14, 17);
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(35, 129, 132);
    style.visuals.hyperlink_color = accent();
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    ctx.set_global_style(style);
}

fn toolbar_button(ui: &mut egui::Ui, label: &str, shortcut: &str, mut action: impl FnMut()) {
    if ui.button(label).on_hover_text(shortcut).clicked() {
        action();
    }
}

fn panel_header(ui: &mut egui::Ui, label: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).strong().color(soft_text()));
    });
    ui.separator();
}

fn inspector_section(ui: &mut egui::Ui, title: &str, contents: impl FnOnce(&mut egui::Ui)) {
    egui::CollapsingHeader::new(title)
        .default_open(true)
        .show(ui, |ui| {
            ui.group(contents);
        });
}

fn asset_tile(ui: &mut egui::Ui, label: &str, color: egui::Color32) {
    ui.vertical(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(112.0, 70.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(28, 35, 39));
        painter.rect_filled(rect.shrink(10.0), 3.0, color);
        draw_grid(
            &painter,
            rect.shrink(10.0),
            16.0,
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 40),
        );
        ui.add_sized([112.0, 18.0], egui::Label::new(label));
    });
}

fn draw_grid(painter: &egui::Painter, rect: egui::Rect, step: f32, color: egui::Color32) {
    let mut x = rect.left();
    while x <= rect.right() {
        painter.line_segment(
            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
            egui::Stroke::new(1.0, color),
        );
        x += step;
    }

    let mut y = rect.top();
    while y <= rect.bottom() {
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            egui::Stroke::new(1.0, color),
        );
        y += step;
    }
}

fn accent() -> egui::Color32 {
    egui::Color32::from_rgb(44, 198, 194)
}

fn soft_text() -> egui::Color32 {
    egui::Color32::from_rgb(198, 213, 218)
}
