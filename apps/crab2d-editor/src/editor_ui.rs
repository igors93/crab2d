use std::path::PathBuf;

use crate::editor_assets::{EditorTextureCache, ImageAssetCatalog, TextureLookup};
use crab2d_editor::{
    EditorApp, EditorCommand, EditorCommandResult, EntityId, TileCell, TilemapComponent,
    Transform2D, Vec2,
};
use eframe::egui;

pub struct Crab2DEditorUi {
    app: EditorApp,
    selected: Option<EntityId>,
    name_edit: String,
    filter_edit: String,
    tag_edit: String,
    sprite_edit: String,
    textures: EditorTextureCache,
    assets: ImageAssetCatalog,
    active_tool: EditorTool,
    selected_tile_index: u32,
    status: String,
    output: Vec<String>,
    transform_drag: Option<(EntityId, Transform2D)>,
}

impl Crab2DEditorUi {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_style(&cc.egui_ctx);

        let roots = asset_roots();
        let mut app = EditorApp::new("Crab2D Editor");
        app.open_empty_project("Untitled Project");
        let selected = Self::default_selected_node(&app);

        let mut editor = Self {
            app,
            selected,
            name_edit: String::new(),
            filter_edit: String::new(),
            tag_edit: String::new(),
            sprite_edit: String::new(),
            textures: EditorTextureCache::new(roots.clone()),
            assets: ImageAssetCatalog::scan(&roots),
            active_tool: EditorTool::Select,
            selected_tile_index: 0,
            status: "Ready".to_owned(),
            output: vec![
                "[INFO] Crab2D editor started".to_owned(),
                "[INFO] Starter scene loaded".to_owned(),
            ],
            transform_drag: None,
        };
        editor.sync_selected_buffers();
        editor
    }

    fn default_selected_node(app: &EditorApp) -> Option<EntityId> {
        app.scene_nodes()
            .iter()
            .find(|node| node.name == "Player")
            .or_else(|| app.scene_nodes().first())
            .map(|node| node.id)
    }

    fn select_node(&mut self, id: EntityId) {
        if self.selected == Some(id) {
            return;
        }
        self.transform_drag = None;
        self.selected = Some(id);
        self.sync_selected_buffers();
    }

    fn select_default_node(&mut self) {
        self.selected = Self::default_selected_node(&self.app);
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
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.handle_shortcuts(&ctx);
        self.show_top_bar(ui);
        self.show_scene_panel(ui);
        self.show_inspector(ui);
        self.show_bottom_dock(ui);
        self.show_viewport(ui);
    }
}

impl Crab2DEditorUi {
    fn show_top_bar(&mut self, root: &mut egui::Ui) {
        egui::Panel::top("top_bar")
            .exact_size(58.0)
            .show_inside(root, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(8.0);
                    self.show_logo(ui);
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
                        .selectable_label(self.active_tool == EditorTool::Select, "Select")
                        .on_hover_text("Select and inspect nodes")
                        .clicked()
                    {
                        self.active_tool = EditorTool::Select;
                    }
                    if ui
                        .selectable_label(self.active_tool == EditorTool::TileBrush, "Tile Brush")
                        .on_hover_text("Paint the selected tile into the active tilemap")
                        .clicked()
                    {
                        self.active_tool = EditorTool::TileBrush;
                    }
                    if ui
                        .selectable_label(self.active_tool == EditorTool::EraseTile, "Erase")
                        .on_hover_text("Erase tiles from the active tilemap")
                        .clicked()
                    {
                        self.active_tool = EditorTool::EraseTile;
                    }

                    ui.separator();
                    if ui
                        .add(egui::Button::new(
                            egui::RichText::new("Play").color(accent()),
                        ))
                        .clicked()
                    {
                        self.app.preview_procedural_world();
                        self.set_status("Preview generated");
                    }
                    ui.add_enabled(false, egui::Button::new("Pause"));
                    ui.add_enabled(false, egui::Button::new("Stop"));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new(self.status.as_str()).color(soft_text()));
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
            .default_size(245.0)
            .min_size(190.0)
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
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for id in ids {
                        let is_selected = self.selected == Some(id);
                        let response = ui.selectable_label(is_selected, self.node_label(id));
                        if response.clicked() {
                            self.select_node(id);
                        }
                    }
                });

                ui.separator();
                panel_header(ui, "WORLD");
                if let Some(tilemap_id) = self.app.first_tilemap_node() {
                    if let Some(tilemap) = self.app.node_tilemap(tilemap_id) {
                        ui.horizontal(|ui| {
                            ui.weak("Map");
                            ui.monospace(format!(
                                "{} x {}",
                                tilemap.map_size.width, tilemap.map_size.height
                            ));
                        });
                        ui.horizontal(|ui| {
                            ui.weak("Tile");
                            ui.monospace(format!(
                                "{} x {}",
                                tilemap.tile_size.width, tilemap.tile_size.height
                            ));
                        });
                    }
                }
            });
    }

    fn show_inspector(&mut self, root: &mut egui::Ui) {
        egui::Panel::right("inspector_panel")
            .resizable(true)
            .default_size(340.0)
            .min_size(260.0)
            .show_inside(root, |ui| {
                panel_header(ui, "INSPECTOR");

                let Some(entity) = self.selected else {
                    ui.weak("No node selected.");
                    return;
                };

                let Some(node) = self.app.find_node(entity).cloned() else {
                    self.selected = None;
                    return;
                };

                ui.horizontal(|ui| {
                    ui.weak("Node");
                    ui.monospace(format!("#{}", entity.raw()));
                });

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

                self.show_transform_inspector(ui, entity, node.transform);
                self.show_tag_inspector(ui, entity);
                self.show_sprite_inspector(ui, entity);
                self.show_tilemap_inspector(ui, entity);
            });
    }

    fn show_transform_inspector(
        &mut self,
        ui: &mut egui::Ui,
        entity: EntityId,
        transform_before: Transform2D,
    ) {
        let mut transform = transform_before;
        inspector_section(ui, "Transform2D", |ui| {
            let mut changed = false;
            let mut drag_started = false;
            let mut drag_stopped = false;

            egui::Grid::new("transform_editor")
                .num_columns(3)
                .spacing([8.0, 6.0])
                .show(ui, |ui| {
                    ui.weak("Position");
                    drag_value(
                        ui,
                        &mut transform.position.x,
                        1.0,
                        &mut changed,
                        &mut drag_started,
                        &mut drag_stopped,
                    );
                    drag_value(
                        ui,
                        &mut transform.position.y,
                        1.0,
                        &mut changed,
                        &mut drag_started,
                        &mut drag_stopped,
                    );
                    ui.end_row();

                    ui.weak("Scale");
                    drag_value(
                        ui,
                        &mut transform.scale.x,
                        0.05,
                        &mut changed,
                        &mut drag_started,
                        &mut drag_stopped,
                    );
                    drag_value(
                        ui,
                        &mut transform.scale.y,
                        0.05,
                        &mut changed,
                        &mut drag_started,
                        &mut drag_stopped,
                    );
                    ui.end_row();

                    ui.weak("Rotation");
                    let mut degrees = transform.rotation_radians.to_degrees();
                    let response =
                        ui.add(egui::DragValue::new(&mut degrees).speed(1.0).suffix(" deg"));
                    changed |= response.changed();
                    drag_started |= response.drag_started();
                    drag_stopped |= response.drag_stopped();
                    transform.rotation_radians = degrees.to_radians();
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
                }
            }

            if drag_stopped {
                if let Some((eid, before)) = self.transform_drag.take() {
                    let after = self.app.node_transform(eid).unwrap_or(transform);
                    self.app.record_move_node(eid, before, after);
                    self.status = "Transform updated".to_owned();
                }
            }
        });
    }

    fn show_tag_inspector(&mut self, ui: &mut egui::Ui, entity: EntityId) {
        inspector_section(ui, "Tag", |ui| {
            ui.text_edit_singleline(&mut self.tag_edit);
            if ui.button("Apply Tag").clicked() {
                let tag = self.tag_edit.trim().to_owned();
                if !tag.is_empty() {
                    match self
                        .app
                        .execute_command_with_history(EditorCommand::attach_tag(entity, tag))
                    {
                        Ok(_) => self.set_status("Tag applied"),
                        Err(error) => self.set_error(format!("{error}")),
                    }
                }
            }

            if let Some(tag) = self.app.node_tag(entity) {
                ui.weak(format!("Current: {}", tag.tag));
            }
        });
    }

    fn show_sprite_inspector(&mut self, ui: &mut egui::Ui, entity: EntityId) {
        inspector_section(ui, "Sprite", |ui| {
            ui.text_edit_singleline(&mut self.sprite_edit);
            if ui.button("Apply Sprite").clicked() {
                let sprite_path = self.sprite_edit.trim().to_owned();
                if !sprite_path.is_empty() {
                    self.apply_sprite(entity, sprite_path);
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
    }

    fn show_tilemap_inspector(&mut self, ui: &mut egui::Ui, entity: EntityId) {
        if let Some(tilemap) = self.app.node_tilemap(entity) {
            let map_width = tilemap.map_size.width;
            let map_height = tilemap.map_size.height;
            let tile_width = tilemap.tile_size.width;
            let tile_height = tilemap.tile_size.height;
            let layer_count = tilemap.layers.len();

            inspector_section(ui, "Tilemap", |ui| {
                ui.horizontal(|ui| {
                    ui.weak("Map Size");
                    ui.monospace(format!("{map_width} x {map_height}"));
                });
                ui.horizontal(|ui| {
                    ui.weak("Tile Size");
                    ui.monospace(format!("{tile_width} x {tile_height}"));
                });
                ui.horizontal(|ui| {
                    ui.weak("Layers");
                    ui.label(layer_count.to_string());
                });
                if ui.button("Use Tile Brush").clicked() {
                    self.active_tool = EditorTool::TileBrush;
                    self.set_status("Tile Brush active");
                }
            });
        }
    }

    fn show_bottom_dock(&mut self, root: &mut egui::Ui) {
        egui::Panel::bottom("bottom_dock")
            .resizable(true)
            .default_size(190.0)
            .min_size(125.0)
            .show_inside(root, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut self.active_tool,
                        EditorTool::TileBrush,
                        "Tile Palette",
                    );
                    ui.label("Assets");
                    if ui.button("Refresh").clicked() {
                        self.assets = ImageAssetCatalog::scan(&asset_roots());
                        self.set_status("Assets refreshed");
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.weak(format!("{} image(s)", self.assets.images().len()));
                    });
                });
                ui.separator();

                ui.columns(2, |columns| {
                    columns[0].vertical(|ui| {
                        ui.monospace("TILES");
                        ui.horizontal_wrapped(|ui| {
                            for index in 0..16 {
                                let selected = self.selected_tile_index == index;
                                let (rect, response) = ui.allocate_exact_size(
                                    egui::vec2(34.0, 34.0),
                                    egui::Sense::click(),
                                );
                                ui.painter().rect_filled(rect, 3.0, tile_color(index));
                                if selected {
                                    ui.painter().rect_stroke(
                                        rect,
                                        3.0,
                                        egui::Stroke::new(2.0, accent()),
                                        egui::StrokeKind::Inside,
                                    );
                                }
                                if response.clicked() {
                                    self.selected_tile_index = index;
                                    self.active_tool = EditorTool::TileBrush;
                                }
                            }
                        });
                        ui.weak("Click a tile, then paint into the viewport.");
                    });

                    columns[1].vertical(|ui| {
                        ui.monospace("IMAGE ASSETS");
                        if self.assets.is_empty() {
                            ui.weak("No images found in the editor or project asset roots.");
                            return;
                        }

                        let images = self.assets.images().to_vec();
                        egui::ScrollArea::horizontal().show(ui, |ui| {
                            ui.horizontal(|ui| {
                                for image in images {
                                    if self.image_asset_tile(
                                        ui,
                                        &image.display_name,
                                        &image.asset_path,
                                    ) {
                                        if let Some(entity) = self.selected {
                                            self.sprite_edit = image.asset_path.clone();
                                            self.apply_sprite(entity, image.asset_path);
                                        }
                                    }
                                }
                            });
                        });
                    });
                });

                ui.separator();
                ui.horizontal(|ui| {
                    ui.monospace("OUTPUT");
                    for line in self.output.iter().rev().take(3) {
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
    }

    fn show_logo(&mut self, ui: &mut egui::Ui) {
        let (rect, response) = ui.allocate_exact_size(egui::vec2(36.0, 36.0), egui::Sense::hover());
        match self.textures.load(ui.ctx(), "logo.png") {
            TextureLookup::Loaded(texture) => {
                ui.painter().image(
                    texture.id(),
                    rect,
                    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }
            TextureLookup::Failed(error) => {
                draw_fallback_logo(ui.painter(), rect);
                response.on_hover_text(error);
            }
            TextureLookup::Missing => {
                draw_fallback_logo(ui.painter(), rect);
            }
        }
    }

    fn show_viewport(&mut self, root: &mut egui::Ui) {
        let items: Vec<NodeView> = self
            .app
            .scene_nodes()
            .iter()
            .map(|node| NodeView {
                id: node.id,
                name: node.name.clone(),
                transform: node.transform,
                sprite_path: self
                    .app
                    .node_sprite(node.id)
                    .map(|sprite| sprite.sprite_path.clone()),
                has_camera: self.app.node_camera(node.id).is_some(),
                tilemap: self.app.node_tilemap(node.id).cloned(),
            })
            .collect();

        let mut clicked_id = None;
        let mut clicked_name = None;
        let mut paint_request = None;

        egui::CentralPanel::default().show_inside(root, |ui| {
            let available = ui.available_size();
            let (rect, response) = ui.allocate_exact_size(available, egui::Sense::click_and_drag());
            let painter = ui.painter_at(rect);
            let world_to_screen =
                |position: Vec2| rect.center() + egui::vec2(position.x, -position.y);
            let screen_to_world = |position: egui::Pos2| {
                Vec2::new(
                    position.x - rect.center().x,
                    -(position.y - rect.center().y),
                )
            };

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

            let mut hit_rects = Vec::new();

            for item in &items {
                if let Some(tilemap) = &item.tilemap {
                    let tilemap_rect = self.draw_tilemap(
                        ui.ctx(),
                        &painter,
                        rect,
                        &world_to_screen,
                        item,
                        tilemap,
                    );
                    hit_rects.push((item.id, tilemap_rect));
                }
            }

            for item in &items {
                let hit_rect = self.draw_node(ui.ctx(), &painter, &world_to_screen, item);
                hit_rects.push((item.id, hit_rect));
            }

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    match self.active_tool {
                        EditorTool::TileBrush | EditorTool::EraseTile => {
                            paint_request = Some(screen_to_world(pos));
                        }
                        EditorTool::Select => {
                            if let Some((id, _)) =
                                hit_rects.iter().rev().find(|(_, hit)| hit.contains(pos))
                            {
                                clicked_id = Some(*id);
                                clicked_name = items
                                    .iter()
                                    .find(|item| item.id == *id)
                                    .map(|item| item.name.clone());
                            }
                        }
                    }
                }
            }

            painter.text(
                rect.left_top() + egui::vec2(14.0, 12.0),
                egui::Align2::LEFT_TOP,
                format!("2D Viewport - {:?}", self.active_tool),
                egui::FontId::proportional(14.0),
                soft_text(),
            );
            painter.text(
                rect.right_bottom() - egui::vec2(14.0, 12.0),
                egui::Align2::RIGHT_BOTTOM,
                format!("Selected Tile: {}", self.selected_tile_index),
                egui::FontId::monospace(12.0),
                egui::Color32::from_rgb(130, 145, 150),
            );
        });

        if let Some(world_position) = paint_request {
            self.paint_tile_at(world_position);
        }

        if let Some(id) = clicked_id {
            self.select_node(id);
            if let Some(name) = clicked_name {
                self.set_status(format!("Selected: {name}"));
            }
        }
    }

    fn draw_tilemap(
        &mut self,
        ctx: &egui::Context,
        painter: &egui::Painter,
        viewport_rect: egui::Rect,
        world_to_screen: &dyn Fn(Vec2) -> egui::Pos2,
        item: &NodeView,
        tilemap: &TilemapComponent,
    ) -> egui::Rect {
        let tile_width = tilemap.tile_size.width as f32;
        let tile_height = tilemap.tile_size.height as f32;
        let origin = item.transform.position;
        let map_size = egui::vec2(
            tilemap.map_size.width as f32 * tile_width,
            tilemap.map_size.height as f32 * tile_height,
        );
        let map_center = world_to_screen(Vec2::new(
            origin.x + map_size.x / 2.0,
            origin.y + map_size.y / 2.0,
        ));
        let map_rect = egui::Rect::from_center_size(map_center, map_size);
        let mut texture_error = None;
        let tileset_texture = tilemap.tileset.as_ref().and_then(|tileset| {
            match self.textures.load(ctx, &tileset.image_path) {
                TextureLookup::Loaded(texture) => {
                    Some((texture.id(), tileset.columns, tileset.rows))
                }
                TextureLookup::Failed(error) => {
                    texture_error = Some(error);
                    None
                }
                TextureLookup::Missing => None,
            }
        });

        for visible in tilemap.visible_tiles() {
            let world_center = Vec2::new(
                origin.x + visible.x as f32 * tile_width + tile_width / 2.0,
                origin.y + visible.y as f32 * tile_height + tile_height / 2.0,
            );
            let tile_rect = egui::Rect::from_center_size(
                world_to_screen(world_center),
                egui::vec2(tile_width, tile_height),
            );

            if !viewport_rect.intersects(tile_rect) {
                continue;
            }

            if let Some((texture_id, columns, rows)) = tileset_texture {
                let uv = tile_uv(visible.cell.tile_index, columns, rows);
                painter.image(
                    texture_id,
                    tile_rect,
                    uv,
                    egui::Color32::from_rgba_unmultiplied(
                        visible.cell.tint_rgba[0],
                        visible.cell.tint_rgba[1],
                        visible.cell.tint_rgba[2],
                        visible.cell.tint_rgba[3],
                    ),
                );
            } else {
                painter.rect_filled(tile_rect, 0.0, tile_color(visible.cell.tile_index));
            }
        }

        painter.rect_stroke(
            map_rect,
            0.0,
            egui::Stroke::new(
                if self.selected == Some(item.id) {
                    2.0
                } else {
                    1.0
                },
                if self.selected == Some(item.id) {
                    accent()
                } else {
                    egui::Color32::from_rgb(82, 110, 118)
                },
            ),
            egui::StrokeKind::Inside,
        );

        if let Some(error) = texture_error {
            painter.text(
                map_rect.left_top() + egui::vec2(8.0, 8.0),
                egui::Align2::LEFT_TOP,
                format!("Tileset load failed: {error}"),
                egui::FontId::monospace(11.0),
                egui::Color32::from_rgb(245, 112, 112),
            );
        }

        map_rect
    }

    fn draw_node(
        &mut self,
        ctx: &egui::Context,
        painter: &egui::Painter,
        world_to_screen: &dyn Fn(Vec2) -> egui::Pos2,
        item: &NodeView,
    ) -> egui::Rect {
        let center = world_to_screen(item.transform.position);
        let is_selected = self.selected == Some(item.id);
        let color = if is_selected {
            accent()
        } else if item.has_camera {
            egui::Color32::from_rgb(120, 148, 255)
        } else if item.sprite_path.is_some() {
            egui::Color32::from_rgb(102, 198, 91)
        } else {
            egui::Color32::from_rgb(214, 166, 84)
        };

        let hit_rect = if let Some(sprite_path) = item.sprite_path.as_deref() {
            match self.textures.load(ctx, sprite_path) {
                TextureLookup::Loaded(texture) => {
                    let size = texture.size_vec2();
                    let size = egui::vec2(
                        (size.x * item.transform.scale.x.abs().max(0.1)).clamp(12.0, 256.0),
                        (size.y * item.transform.scale.y.abs().max(0.1)).clamp(12.0, 256.0),
                    );
                    let rect = egui::Rect::from_center_size(center, size);
                    painter.image(
                        texture.id(),
                        rect,
                        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                    rect
                }
                TextureLookup::Failed(error) => {
                    let rect = egui::Rect::from_center_size(center, egui::vec2(42.0, 42.0));
                    draw_missing_texture_marker(painter, rect, "!");
                    painter.text(
                        rect.center_bottom() + egui::vec2(0.0, 3.0),
                        egui::Align2::CENTER_TOP,
                        "asset failed",
                        egui::FontId::monospace(10.0),
                        egui::Color32::from_rgb(245, 112, 112),
                    );
                    painter.text(
                        rect.center_top() - egui::vec2(0.0, 4.0),
                        egui::Align2::CENTER_BOTTOM,
                        compact_error(&error),
                        egui::FontId::monospace(9.0),
                        egui::Color32::from_rgb(245, 170, 170),
                    );
                    rect
                }
                TextureLookup::Missing => {
                    let rect = egui::Rect::from_center_size(center, egui::vec2(42.0, 42.0));
                    draw_node_marker(painter, center, color);
                    rect
                }
            }
        } else {
            let rect = egui::Rect::from_center_size(center, egui::vec2(34.0, 34.0));
            draw_node_marker(painter, center, color);
            rect
        };

        if is_selected {
            painter.rect_stroke(
                hit_rect.expand(4.0),
                4.0,
                egui::Stroke::new(2.0, accent()),
                egui::StrokeKind::Inside,
            );
        }
        painter.text(
            hit_rect.center_bottom() + egui::vec2(0.0, 6.0),
            egui::Align2::CENTER_TOP,
            &item.name,
            egui::FontId::monospace(11.0),
            soft_text(),
        );
        hit_rect
    }

    fn paint_tile_at(&mut self, world_position: Vec2) {
        let entity = self
            .selected
            .filter(|id| self.app.node_tilemap(*id).is_some())
            .or_else(|| self.app.first_tilemap_node());
        let Some(entity) = entity else {
            self.set_error("No tilemap node available");
            return;
        };

        let Some(node) = self.app.find_node(entity).cloned() else {
            self.set_error("Tilemap node was not found");
            return;
        };
        let Some(tilemap) = self.app.node_tilemap(entity).cloned() else {
            self.set_error("Tilemap component was not found");
            return;
        };

        let local_x = world_position.x - node.transform.position.x;
        let local_y = world_position.y - node.transform.position.y;
        if local_x < 0.0 || local_y < 0.0 {
            return;
        }

        let x = (local_x / tilemap.tile_size.width as f32).floor() as u32;
        let y = (local_y / tilemap.tile_size.height as f32).floor() as u32;
        if x >= tilemap.map_size.width || y >= tilemap.map_size.height {
            return;
        }

        let tile = match self.active_tool {
            EditorTool::TileBrush => Some(TileCell::new(self.selected_tile_index)),
            EditorTool::EraseTile => None,
            EditorTool::Select => return,
        };

        match self
            .app
            .execute_command_with_history(EditorCommand::set_tile(entity, "Ground", x, y, tile))
        {
            Ok(_) => self.set_status(format!("Tile ({x}, {y}) updated")),
            Err(error) => self.set_error(format!("{error}")),
        }
    }

    fn image_asset_tile(&mut self, ui: &mut egui::Ui, label: &str, asset_path: &str) -> bool {
        let mut clicked = false;
        ui.vertical(|ui| {
            let (rect, response) =
                ui.allocate_exact_size(egui::vec2(112.0, 76.0), egui::Sense::click());
            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(28, 35, 39));
            let mut load_error = None;
            match self.textures.load(ui.ctx(), asset_path) {
                TextureLookup::Loaded(texture) => {
                    let image_rect = rect.shrink(8.0);
                    painter.image(
                        texture.id(),
                        image_rect,
                        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                }
                TextureLookup::Failed(error) => {
                    load_error = Some(error);
                    draw_missing_texture_marker(&painter, rect.shrink(12.0), "!");
                    painter.text(
                        rect.center_bottom() - egui::vec2(0.0, 12.0),
                        egui::Align2::CENTER_BOTTOM,
                        "Load failed",
                        egui::FontId::monospace(10.0),
                        egui::Color32::from_rgb(245, 112, 112),
                    );
                }
                TextureLookup::Missing => {
                    painter.rect_filled(
                        rect.shrink(12.0),
                        3.0,
                        egui::Color32::from_rgb(74, 46, 48),
                    );
                }
            }
            clicked = response.clicked();
            if let Some(error) = load_error {
                response.on_hover_text(error);
            }
            ui.add_sized([112.0, 18.0], egui::Label::new(label));
        });
        clicked
    }

    fn apply_sprite(&mut self, entity: EntityId, sprite_path: String) {
        match self
            .app
            .execute_command_with_history(EditorCommand::attach_sprite(entity, sprite_path))
        {
            Ok(_) => {
                self.sync_selected_buffers();
                self.set_status("Sprite applied");
            }
            Err(error) => self.set_error(format!("{error}")),
        }
    }

    fn node_label(&self, id: EntityId) -> String {
        let name = self
            .app
            .find_node(id)
            .map(|node| node.name.as_str())
            .unwrap_or("?");

        if self.app.node_tilemap(id).is_some() {
            format!("tilemap {name}")
        } else if self.app.node_camera(id).is_some() {
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

#[derive(Debug, Clone, PartialEq)]
struct NodeView {
    id: EntityId,
    name: String,
    transform: Transform2D,
    sprite_path: Option<String>,
    has_camera: bool,
    tilemap: Option<TilemapComponent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditorTool {
    Select,
    TileBrush,
    EraseTile,
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

fn drag_value(
    ui: &mut egui::Ui,
    value: &mut f32,
    speed: f64,
    changed: &mut bool,
    drag_started: &mut bool,
    drag_stopped: &mut bool,
) {
    let response = ui.add(egui::DragValue::new(value).speed(speed));
    *changed |= response.changed();
    *drag_started |= response.drag_started();
    *drag_stopped |= response.drag_stopped();
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

fn draw_node_marker(painter: &egui::Painter, center: egui::Pos2, color: egui::Color32) {
    let node_rect = egui::Rect::from_center_size(center, egui::vec2(34.0, 34.0));
    painter.rect_filled(node_rect, 4.0, color.gamma_multiply(0.65));
    painter.rect_stroke(
        node_rect,
        4.0,
        egui::Stroke::new(1.5, color),
        egui::StrokeKind::Inside,
    );
}

fn draw_missing_texture_marker(painter: &egui::Painter, rect: egui::Rect, label: &str) {
    painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(74, 46, 48));
    painter.rect_stroke(
        rect,
        4.0,
        egui::Stroke::new(1.5, egui::Color32::from_rgb(245, 112, 112)),
        egui::StrokeKind::Inside,
    );
    painter.line_segment(
        [rect.left_top(), rect.right_bottom()],
        egui::Stroke::new(1.0, egui::Color32::from_rgb(245, 112, 112)),
    );
    painter.line_segment(
        [rect.right_top(), rect.left_bottom()],
        egui::Stroke::new(1.0, egui::Color32::from_rgb(245, 112, 112)),
    );
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(18.0),
        egui::Color32::from_rgb(255, 220, 220),
    );
}

fn draw_fallback_logo(painter: &egui::Painter, rect: egui::Rect) {
    painter.circle_filled(rect.center(), 16.0, accent());
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "C",
        egui::FontId::proportional(18.0),
        egui::Color32::from_rgb(10, 18, 20),
    );
}

fn compact_error(error: &str) -> String {
    const MAX_LEN: usize = 42;
    if error.chars().count() <= MAX_LEN {
        return error.to_owned();
    }

    let mut compact = error.chars().take(MAX_LEN).collect::<String>();
    compact.push_str("...");
    compact
}

fn tile_uv(tile_index: u32, columns: u32, rows: u32) -> egui::Rect {
    let columns = columns.max(1);
    let rows = rows.max(1);
    let tile_index = tile_index % (columns * rows);
    let column = tile_index % columns;
    let row = tile_index / columns;
    let min = egui::pos2(column as f32 / columns as f32, row as f32 / rows as f32);
    let max = egui::pos2(
        (column + 1) as f32 / columns as f32,
        (row + 1) as f32 / rows as f32,
    );
    egui::Rect::from_min_max(min, max)
}

fn tile_color(tile_index: u32) -> egui::Color32 {
    match tile_index % 8 {
        0 => egui::Color32::from_rgb(76, 132, 58),
        1 => egui::Color32::from_rgb(94, 154, 68),
        2 => egui::Color32::from_rgb(151, 128, 82),
        3 => egui::Color32::from_rgb(86, 94, 74),
        4 => egui::Color32::from_rgb(52, 105, 150),
        5 => egui::Color32::from_rgb(122, 82, 151),
        6 => egui::Color32::from_rgb(172, 106, 52),
        _ => egui::Color32::from_rgb(160, 180, 190),
    }
}

fn asset_roots() -> Vec<PathBuf> {
    let mut roots = vec![PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")];
    if let Ok(current_dir) = std::env::current_dir() {
        roots.push(current_dir.join("assets"));
    }
    roots
}

fn accent() -> egui::Color32 {
    egui::Color32::from_rgb(44, 198, 194)
}

fn soft_text() -> egui::Color32 {
    egui::Color32::from_rgb(198, 213, 218)
}
