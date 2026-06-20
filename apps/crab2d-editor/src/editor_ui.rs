use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::Command;

use crate::editor_assets::{EditorTextureCache, ImageAsset, ImageAssetCatalog, TextureLookup};
use crate::editor_theme::{configure_style, theme, tile_color};
use crate::editor_widgets::{self as widgets, StatusTone};
use crab2d_editor::{
    default_tilemap, Camera2DComponent, CameraFollowComponent, Collider2DComponent, EditorApp,
    EditorCommand, EditorCommandResult, EditorComponentKind, EntityId, GameplayPreset,
    PlayerControllerComponent, ProjectTemplate, TileCell, TilemapComponent, Transform2D,
    TriggerComponent, Vec2, Velocity2DComponent,
};
use eframe::egui;

pub struct Crab2DEditorUi {
    app: EditorApp,
    selected: Option<EntityId>,
    name_edit: String,
    scene_filter_edit: String,
    asset_filter_edit: String,
    tag_edit: String,
    sprite_edit: String,
    velocity_x_edit: f32,
    velocity_y_edit: f32,
    collider_width_edit: f32,
    collider_height_edit: f32,
    collider_sensor_edit: bool,
    controller_speed_edit: f32,
    controller_enabled_edit: bool,
    camera_follow_target_edit: String,
    camera_follow_smoothing_edit: f32,
    camera_follow_enabled_edit: bool,
    trigger_name_edit: String,
    trigger_once_edit: bool,
    tile_collision_edit: String,
    project_name_edit: String,
    project_path_edit: String,
    open_path_edit: String,
    save_as_path_edit: String,
    selected_template: ProjectTemplate,
    active_dialog: Option<ProjectDialog>,
    textures: EditorTextureCache,
    assets: ImageAssetCatalog,
    active_tool: EditorTool,
    selected_tile_index: u32,
    left_panel_tab: LeftPanelTab,
    bottom_tab: BottomDockTab,
    asset_tab: AssetBrowserTab,
    selected_asset_path: Option<String>,
    status: String,
    status_tone: StatusTone,
    output: Vec<String>,
    last_asset_error: Option<String>,
    transform_drag: Option<(EntityId, Transform2D)>,
}

impl Crab2DEditorUi {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_style(&cc.egui_ctx);

        let mut app = EditorApp::new("Crab2D Editor");
        app.open_empty_project("Untitled Project");
        let roots = asset_roots(app.asset_roots());
        let selected = Self::default_selected_node(&app);

        let mut editor = Self {
            app,
            selected,
            name_edit: String::new(),
            scene_filter_edit: String::new(),
            asset_filter_edit: String::new(),
            tag_edit: String::new(),
            sprite_edit: String::new(),
            velocity_x_edit: 0.0,
            velocity_y_edit: 0.0,
            collider_width_edit: 24.0,
            collider_height_edit: 24.0,
            collider_sensor_edit: false,
            controller_speed_edit: PlayerControllerComponent::DEFAULT_MOVE_SPEED,
            controller_enabled_edit: true,
            camera_follow_target_edit: String::new(),
            camera_follow_smoothing_edit: 0.0,
            camera_follow_enabled_edit: true,
            trigger_name_edit: "trigger".to_owned(),
            trigger_once_edit: false,
            tile_collision_edit: String::new(),
            project_name_edit: "UntitledProject".to_owned(),
            project_path_edit: default_project_path("UntitledProject"),
            open_path_edit: "project.crab2d.json".to_owned(),
            save_as_path_edit: "project.crab2d.json".to_owned(),
            selected_template: ProjectTemplate::TopDownStarter,
            active_dialog: None,
            textures: EditorTextureCache::new(roots.clone()),
            assets: ImageAssetCatalog::scan(&roots),
            active_tool: EditorTool::Select,
            selected_tile_index: 0,
            left_panel_tab: LeftPanelTab::Scene,
            bottom_tab: BottomDockTab::TilePalette,
            asset_tab: AssetBrowserTab::Images,
            selected_asset_path: None,
            status: "Ready".to_owned(),
            status_tone: StatusTone::Info,
            output: vec![
                "[INFO] Crab2D editor started".to_owned(),
                "[INFO] Starter scene loaded".to_owned(),
            ],
            last_asset_error: None,
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
            self.velocity_x_edit = 0.0;
            self.velocity_y_edit = 0.0;
            self.collider_width_edit = 24.0;
            self.collider_height_edit = 24.0;
            self.collider_sensor_edit = false;
            self.controller_speed_edit = PlayerControllerComponent::DEFAULT_MOVE_SPEED;
            self.controller_enabled_edit = true;
            self.camera_follow_target_edit.clear();
            self.camera_follow_smoothing_edit = 0.0;
            self.camera_follow_enabled_edit = true;
            self.trigger_name_edit = "trigger".to_owned();
            self.trigger_once_edit = false;
            self.tile_collision_edit.clear();
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
        let velocity = self
            .app
            .node_velocity(id)
            .copied()
            .unwrap_or(Velocity2DComponent::ZERO);
        self.velocity_x_edit = velocity.linear.x;
        self.velocity_y_edit = velocity.linear.y;

        let collider = self
            .app
            .node_collider(id)
            .copied()
            .unwrap_or_else(|| Collider2DComponent::rectangle(Vec2::new(24.0, 24.0)));
        self.collider_width_edit = collider.half_extents.x * 2.0;
        self.collider_height_edit = collider.half_extents.y * 2.0;
        self.collider_sensor_edit = collider.is_sensor;

        let controller = self
            .app
            .node_player_controller(id)
            .copied()
            .unwrap_or_default();
        self.controller_speed_edit = controller.move_speed;
        self.controller_enabled_edit = controller.enabled;

        let follow = self
            .app
            .node_camera_follow(id)
            .copied()
            .unwrap_or_else(|| CameraFollowComponent::new(id));
        self.camera_follow_target_edit = follow.target.raw().to_string();
        self.camera_follow_smoothing_edit = follow.smoothing;
        self.camera_follow_enabled_edit = follow.enabled;

        let trigger = self
            .app
            .node_trigger(id)
            .cloned()
            .unwrap_or_else(|| TriggerComponent::new("trigger"));
        self.trigger_name_edit = trigger.name;
        self.trigger_once_edit = trigger.once;

        self.tile_collision_edit = self
            .app
            .node_tilemap(id)
            .map(|tilemap| format_solid_tiles(&tilemap.collision.solid_tiles))
            .unwrap_or_default();
    }

    fn set_status(&mut self, message: impl Into<String>) {
        self.push_status(message, StatusTone::Info);
    }

    fn set_success(&mut self, message: impl Into<String>) {
        self.push_status(message, StatusTone::Success);
    }

    fn set_error(&mut self, message: impl Into<String>) {
        self.push_status(message, StatusTone::Error);
    }

    fn push_status(&mut self, message: impl Into<String>, tone: StatusTone) {
        let message = message.into();
        self.status = message.clone();
        self.status_tone = tone;
        if !message.is_empty() {
            self.output
                .push(format!("[{}] {message}", output_level(tone)));
            trim_output(&mut self.output);
        }
    }

    fn report_asset_error(&mut self, message: String) {
        if self.last_asset_error.as_deref() == Some(message.as_str()) {
            return;
        }
        self.last_asset_error = Some(message.clone());
        self.push_status(message, StatusTone::Error);
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let (want_undo, want_redo, want_save, want_open) = ctx.input(|input| {
            let undo =
                input.key_pressed(egui::Key::Z) && input.modifiers.ctrl && !input.modifiers.shift;
            let redo =
                (input.key_pressed(egui::Key::Z) && input.modifiers.ctrl && input.modifiers.shift)
                    || (input.key_pressed(egui::Key::Y) && input.modifiers.ctrl);
            let save = input.key_pressed(egui::Key::S) && input.modifiers.ctrl;
            let open = input.key_pressed(egui::Key::O) && input.modifiers.ctrl;
            (undo, redo, save, open)
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
        if want_open {
            self.open_project_dialog();
        }
    }

    fn new_project(&mut self) {
        self.project_name_edit = "UntitledProject".to_owned();
        self.project_path_edit = default_project_path(&self.project_name_edit);
        self.selected_template = ProjectTemplate::TopDownStarter;
        self.active_dialog = Some(ProjectDialog::NewProject);
    }

    fn open_project_dialog(&mut self) {
        if let Some(path) = self.app.project_path() {
            self.open_path_edit = path.display().to_string();
        }
        self.active_dialog = Some(ProjectDialog::OpenProject);
    }

    fn save_as_dialog(&mut self) {
        self.save_as_path_edit = self
            .app
            .project_path()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| default_project_file_path(self.app.project_name()));
        self.active_dialog = Some(ProjectDialog::SaveAs);
    }

    fn create_new_project_from_dialog(&mut self) {
        let name = self.project_name_edit.trim().to_owned();
        if name.is_empty() {
            self.set_error("Project name cannot be empty");
            return;
        }

        let path = if self.project_path_edit.trim().is_empty() {
            default_project_path(&name)
        } else {
            self.project_path_edit.trim().to_owned()
        };

        match self
            .app
            .new_project(name.clone(), PathBuf::from(&path), self.selected_template)
        {
            Ok(project_path) => {
                self.scene_filter_edit.clear();
                self.asset_filter_edit.clear();
                self.selected_asset_path = None;
                self.last_asset_error = None;
                self.select_default_node();
                self.refresh_asset_roots();
                self.active_dialog = None;
                self.set_success(format!("Created: {}", project_path.display()));
            }
            Err(error) => self.set_error(format!("New project failed: {error}")),
        }
    }

    fn open_project_from_dialog(&mut self) {
        let path = self.open_path_edit.trim().to_owned();
        if path.is_empty() {
            self.set_error("Project path cannot be empty");
            return;
        }

        match self.app.load_project(&path) {
            Ok(()) => {
                self.select_default_node();
                self.refresh_asset_roots();
                self.active_dialog = None;
                self.set_success(format!("Opened: {path}"));
            }
            Err(error) => self.set_error(format!("Open failed: {error}")),
        }
    }

    fn save_as_from_dialog(&mut self) {
        let path = self.save_as_path_edit.trim();
        if path.is_empty() {
            self.set_error("Save path cannot be empty");
            return;
        }

        match self.app.save_project_as(path) {
            Ok(project_path) => {
                self.refresh_asset_roots();
                self.active_dialog = None;
                self.set_success(format!("Saved: {}", project_path.display()));
            }
            Err(error) => self.set_error(format!("Save As failed: {error}")),
        }
    }

    fn refresh_asset_roots(&mut self) {
        let roots = asset_roots(self.app.asset_roots());
        self.textures.set_asset_roots(roots.clone());
        self.assets = ImageAssetCatalog::scan(&roots);
    }

    fn save_project(&mut self) {
        match self.app.save_project() {
            Ok(path) => self.set_success(format!("Saved: {}", path.display())),
            Err(error) => {
                self.set_error(format!("Save needs a project file: {error}"));
                self.save_as_dialog();
            }
        }
    }

    fn play_project(&mut self) {
        if self.app.project_path().is_none() {
            self.set_error("Save the project before running");
            self.save_as_dialog();
            return;
        }

        if self.app.is_dirty() {
            self.active_dialog = Some(ProjectDialog::SaveBeforePlay);
            return;
        }

        self.launch_runtime();
    }

    fn save_and_play(&mut self) {
        match self.app.save_project() {
            Ok(path) => {
                self.set_success(format!("Saved: {}", path.display()));
                self.active_dialog = None;
                self.launch_runtime();
            }
            Err(error) => {
                self.set_error(format!("Save before Play failed: {error}"));
                self.save_as_dialog();
            }
        }
    }

    fn launch_runtime(&mut self) {
        let Some(project_path) = self.app.project_path().map(|path| path.to_path_buf()) else {
            self.set_error("Runtime needs a saved project path");
            return;
        };

        let mut command = Command::new("cargo");
        command
            .args(["run", "-p", "crab2d-runtime-app", "--"])
            .arg(&project_path)
            .current_dir(workspace_root());

        match command.spawn() {
            Ok(_) => self.set_success(format!("Runtime started: {}", project_path.display())),
            Err(error) => self.set_error(format!("Runtime failed to start: {error}")),
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
                self.set_success("Node created");
            }
            Ok(EditorCommandResult::None) => {}
            Err(error) => self.set_error(format!("{error}")),
        }
    }
}

impl eframe::App for Crab2DEditorUi {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(
            self.app.project_session().display_title(),
        ));
        self.handle_shortcuts(&ctx);
        self.show_top_bar(ui);
        self.show_scene_panel(ui);
        self.show_inspector(ui);
        self.show_bottom_dock(ui);
        self.show_viewport(ui);
        self.show_project_dialogs(&ctx);
    }
}

impl Crab2DEditorUi {
    fn show_top_bar(&mut self, root: &mut egui::Ui) {
        let theme = theme();
        egui::Panel::top("top_bar")
            .exact_size(theme.sizing.top_bar_height)
            .frame(
                egui::Frame::new()
                    .fill(theme.colors.panel_bg)
                    .stroke(egui::Stroke::new(1.0, theme.colors.border))
                    .inner_margin(egui::Margin::symmetric(10, 8)),
            )
            .show_inside(root, |ui| {
                ui.horizontal_centered(|ui| {
                    self.show_logo(ui);
                    ui.add_space(6.0);
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Crab2D Editor")
                                    .strong()
                                    .size(15.0)
                                    .color(theme.colors.text),
                            );
                            ui.add_space(2.0);
                            widgets::chip(ui, "v0.2.0", widgets::StatusTone::Info);
                        });
                        ui.label(
                            egui::RichText::new(self.app.project_session().display_title())
                                .size(11.0)
                                .color(theme.colors.text_secondary),
                        );
                        let path = self
                            .app
                            .project_path()
                            .map(|path| path.display().to_string())
                            .unwrap_or_else(|| "No project file".to_owned());
                        ui.label(
                            egui::RichText::new(format!(
                                "{} | {}",
                                self.app.project_session().status_label(),
                                truncate_text(&path, 40)
                            ))
                            .size(10.0)
                            .color(theme.colors.text_muted),
                        );
                    });

                    ui.add_space(theme.spacing.lg);
                    widgets::toolbar_group(ui, "PROJECT", |ui| {
                        if widgets::toolbar_button(ui, "New", "Create a new project", true, false)
                            .clicked()
                        {
                            self.new_project();
                        }
                        if widgets::toolbar_button(ui, "Open", "Open project file", true, false)
                            .clicked()
                        {
                            self.open_project_dialog();
                        }
                        if widgets::toolbar_button(ui, "Save", "Save project", true, false)
                            .clicked()
                        {
                            self.save_project();
                        }
                        if widgets::toolbar_button(ui, "Save As", "Save project as", true, false)
                            .clicked()
                        {
                            self.save_as_dialog();
                        }
                    });
                    ui.separator();

                    widgets::toolbar_group(ui, "EDIT", |ui| {
                        if widgets::toolbar_button(
                            ui,
                            "Undo",
                            "Undo last command",
                            self.app.can_undo(),
                            false,
                        )
                        .clicked()
                        {
                            self.undo();
                        }
                        if widgets::toolbar_button(
                            ui,
                            "Redo",
                            "Redo last command",
                            self.app.can_redo(),
                            false,
                        )
                        .clicked()
                        {
                            self.redo();
                        }
                    });
                    ui.separator();

                    widgets::toolbar_group(ui, "RUN", |ui| {
                        if widgets::play_button(ui, "▶ Play", true).clicked() {
                            self.play_project();
                        }
                        widgets::toolbar_button(ui, "⏸ Pause", "Pause preview", false, false);
                        widgets::toolbar_button(ui, "⏹ Stop", "Stop preview", false, false);
                    });
                    ui.separator();

                    widgets::toolbar_group(ui, "TOOLS", |ui| {
                        self.show_tool_button(ui, EditorTool::Select, "Select", "Select nodes");
                        self.show_tool_button(
                            ui,
                            EditorTool::TileBrush,
                            "Brush",
                            "Paint selected tile",
                        );
                        self.show_tool_button(ui, EditorTool::EraseTile, "Erase", "Erase tiles");
                    });
                    ui.separator();

                    widgets::toolbar_group(ui, "BUILD", |ui| {
                        widgets::toolbar_button(ui, "Export", "Build and export", false, false);
                    });
                    widgets::toolbar_group(ui, "PLUGINS", |ui| {
                        widgets::toolbar_button(ui, "Market", "Plugin market", false, false);
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        widgets::chip(ui, "Rust", StatusTone::Info);
                        ui.label(
                            egui::RichText::new("FPS: 60")
                                .size(10.0)
                                .color(theme.colors.text_muted),
                        );
                    });
                });
            });
    }

    fn show_tool_button(
        &mut self,
        ui: &mut egui::Ui,
        tool: EditorTool,
        label: &str,
        tooltip: &str,
    ) {
        if widgets::toolbar_button(ui, label, tooltip, true, self.active_tool == tool).clicked() {
            self.active_tool = tool;
            self.set_status(format!("{label} tool active"));
        }
    }

    fn show_project_dialogs(&mut self, ctx: &egui::Context) {
        let Some(dialog) = self.active_dialog else {
            return;
        };

        match dialog {
            ProjectDialog::NewProject => {
                egui::Window::new("New Project")
                    .collapsible(false)
                    .resizable(false)
                    .default_width(430.0)
                    .show(ctx, |ui| {
                        widgets::property_row(ui, "Name", |ui| {
                            let response = ui.add_sized(
                                [ui.available_width().max(120.0), 24.0],
                                egui::TextEdit::singleline(&mut self.project_name_edit),
                            );
                            if response.changed() && !self.project_name_edit.trim().is_empty() {
                                self.project_path_edit =
                                    default_project_path(self.project_name_edit.trim());
                            }
                        });
                        widgets::property_row(ui, "Folder", |ui| {
                            ui.add_sized(
                                [ui.available_width().max(120.0), 24.0],
                                egui::TextEdit::singleline(&mut self.project_path_edit),
                            );
                        });
                        widgets::property_row(ui, "Template", |ui| {
                            ui.vertical(|ui| {
                                for template in ProjectTemplate::ALL {
                                    ui.radio_value(
                                        &mut self.selected_template,
                                        template,
                                        template.label(),
                                    );
                                }
                            });
                        });
                        ui.separator();
                        ui.horizontal(|ui| {
                            if widgets::toolbar_button(ui, "Create", "Create project", true, false)
                                .clicked()
                            {
                                self.create_new_project_from_dialog();
                            }
                            if widgets::toolbar_button(ui, "Cancel", "Close dialog", true, false)
                                .clicked()
                            {
                                self.active_dialog = None;
                            }
                        });
                    });
            }
            ProjectDialog::OpenProject => {
                egui::Window::new("Open Project")
                    .collapsible(false)
                    .resizable(false)
                    .default_width(460.0)
                    .show(ctx, |ui| {
                        widgets::property_row(ui, "Project Path", |ui| {
                            ui.add_sized(
                                [ui.available_width().max(160.0), 24.0],
                                egui::TextEdit::singleline(&mut self.open_path_edit),
                            );
                        });
                        ui.separator();
                        ui.horizontal(|ui| {
                            if widgets::toolbar_button(ui, "Open", "Open project", true, false)
                                .clicked()
                            {
                                self.open_project_from_dialog();
                            }
                            if widgets::toolbar_button(ui, "Cancel", "Close dialog", true, false)
                                .clicked()
                            {
                                self.active_dialog = None;
                            }
                        });
                    });
            }
            ProjectDialog::SaveAs => {
                egui::Window::new("Save As")
                    .collapsible(false)
                    .resizable(false)
                    .default_width(460.0)
                    .show(ctx, |ui| {
                        widgets::property_row(ui, "Project Path", |ui| {
                            ui.add_sized(
                                [ui.available_width().max(160.0), 24.0],
                                egui::TextEdit::singleline(&mut self.save_as_path_edit),
                            );
                        });
                        ui.separator();
                        ui.horizontal(|ui| {
                            if widgets::toolbar_button(ui, "Save", "Save project", true, false)
                                .clicked()
                            {
                                self.save_as_from_dialog();
                            }
                            if widgets::toolbar_button(ui, "Cancel", "Close dialog", true, false)
                                .clicked()
                            {
                                self.active_dialog = None;
                            }
                        });
                    });
            }
            ProjectDialog::SaveBeforePlay => {
                egui::Window::new("Save Before Running")
                    .collapsible(false)
                    .resizable(false)
                    .default_width(360.0)
                    .show(ctx, |ui| {
                        ui.label("The current project has unsaved changes.");
                        ui.separator();
                        ui.horizontal(|ui| {
                            if widgets::toolbar_button(
                                ui,
                                "Save & Play",
                                "Save and run",
                                true,
                                false,
                            )
                            .clicked()
                            {
                                self.save_and_play();
                            }
                            if widgets::toolbar_button(
                                ui,
                                "Run Saved",
                                "Run the last saved version",
                                self.app.project_path().is_some(),
                                false,
                            )
                            .clicked()
                            {
                                self.active_dialog = None;
                                self.launch_runtime();
                            }
                            if widgets::toolbar_button(ui, "Cancel", "Close dialog", true, false)
                                .clicked()
                            {
                                self.active_dialog = None;
                            }
                        });
                    });
            }
        }
    }

    fn show_scene_panel(&mut self, root: &mut egui::Ui) {
        let theme = theme();
        egui::Panel::left("scene_panel")
            .resizable(true)
            .default_size(theme.sizing.left_panel_width)
            .min_size(210.0)
            .frame(widgets::panel_frame())
            .show_inside(root, |ui| {
                widgets::panel_header(ui, "SCENE", Some("2D"));
                ui.add_space(theme.spacing.xs);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = theme.spacing.xs;
                    if widgets::segment_button(
                        ui,
                        "Scene",
                        self.left_panel_tab == LeftPanelTab::Scene,
                    )
                    .clicked()
                    {
                        self.left_panel_tab = LeftPanelTab::Scene;
                    }
                    if widgets::segment_button(
                        ui,
                        "Library",
                        self.left_panel_tab == LeftPanelTab::Library,
                    )
                    .clicked()
                    {
                        self.left_panel_tab = LeftPanelTab::Library;
                    }
                });
                ui.add_space(theme.spacing.sm);

                match self.left_panel_tab {
                    LeftPanelTab::Scene => self.show_scene_hierarchy(ui),
                    LeftPanelTab::Library => self.show_library_panel(ui),
                }
            });
    }

    fn show_scene_hierarchy(&mut self, ui: &mut egui::Ui) {
        let theme = theme();
        ui.horizontal(|ui| {
            let search_width =
                (ui.available_width() - theme.sizing.icon_button_size - 8.0).max(80.0);
            ui.add_sized(
                [search_width, 26.0],
                egui::TextEdit::singleline(&mut self.scene_filter_edit).hint_text("Filter nodes"),
            );
            if widgets::icon_button(ui, "+", "Add node", true).clicked() {
                self.create_node();
            }
        });

        ui.add_space(theme.spacing.sm);
        widgets::section_label(ui, "SCENE TREE");
        let filter = self.scene_filter_edit.to_lowercase();
        let ids: Vec<EntityId> = self
            .app
            .scene_nodes()
            .iter()
            .filter(|node| filter.is_empty() || node.name.to_lowercase().contains(filter.as_str()))
            .map(|node| node.id)
            .collect();

        widgets::inset_frame().show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("scene_tree_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for id in ids {
                        self.show_scene_node_row(ui, id);
                    }
                });
        });

        ui.add_space(theme.spacing.md);
        widgets::section_label(ui, "WORLD");
        widgets::inset_frame().show(ui, |ui| {
            if let Some(tilemap_id) = self.app.first_tilemap_node() {
                if let Some(tilemap) = self.app.node_tilemap(tilemap_id) {
                    widgets::property_row(ui, "Map", |ui| {
                        ui.monospace(format!(
                            "{} x {}",
                            tilemap.map_size.width, tilemap.map_size.height
                        ));
                    });
                    widgets::property_row(ui, "Tile", |ui| {
                        ui.monospace(format!(
                            "{} x {}",
                            tilemap.tile_size.width, tilemap.tile_size.height
                        ));
                    });
                    widgets::property_row(ui, "Layers", |ui| {
                        ui.label(tilemap.layers.len().to_string());
                    });
                }
            } else {
                ui.label(egui::RichText::new("No tilemap").color(theme.colors.text_muted));
            }
        });
    }

    fn show_scene_node_row(&mut self, ui: &mut egui::Ui, id: EntityId) {
        let theme = theme();
        let is_selected = self.selected == Some(id);
        let label = self
            .app
            .find_node(id)
            .map(|node| node.name.clone())
            .unwrap_or_else(|| "?".to_owned());
        let (kind, tone) = self.node_kind(id);
        let node_type = self.node_type(id);

        ui.add_space(1.0);
        let available = ui.available_width();
        let row_height = 30.0;
        let (row_rect, row_resp) =
            ui.allocate_exact_size(egui::vec2(available, row_height), egui::Sense::click());

        if ui.is_rect_visible(row_rect) {
            let painter = ui.painter_at(row_rect);
            let fill = if is_selected {
                theme.colors.control_active
            } else if row_resp.hovered() {
                theme.colors.control_hover
            } else {
                egui::Color32::TRANSPARENT
            };
            let stroke_color = if is_selected {
                theme.colors.accent
            } else {
                egui::Color32::TRANSPARENT
            };
            painter.rect_filled(row_rect, theme.radius.sm, fill);
            if is_selected {
                painter.rect_stroke(
                    row_rect,
                    theme.radius.sm,
                    egui::Stroke::new(1.0, stroke_color),
                    egui::StrokeKind::Inside,
                );
                // accent left border
                let left_bar = egui::Rect::from_min_size(
                    row_rect.left_top(),
                    egui::vec2(3.0, row_rect.height()),
                );
                painter.rect_filled(
                    left_bar,
                    egui::CornerRadius {
                        nw: theme.radius.sm,
                        ne: 0,
                        sw: theme.radius.sm,
                        se: 0,
                    },
                    theme.colors.accent,
                );
            }

            // Chip badge
            let chip_text = kind;
            let chip_w = 32.0_f32;
            let chip_rect = egui::Rect::from_min_size(
                row_rect.left_top() + egui::vec2(6.0, (row_height - 16.0) / 2.0),
                egui::vec2(chip_w, 16.0),
            );
            let chip_color = match tone {
                StatusTone::Info => theme.colors.accent,
                StatusTone::Success => theme.colors.success,
                StatusTone::Warning => theme.colors.warning,
                StatusTone::Error => theme.colors.error,
            };
            painter.rect_filled(
                chip_rect,
                theme.radius.xs,
                egui::Color32::from_rgba_unmultiplied(
                    chip_color.r(),
                    chip_color.g(),
                    chip_color.b(),
                    36,
                ),
            );
            painter.rect_stroke(
                chip_rect,
                theme.radius.xs,
                egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgba_unmultiplied(
                        chip_color.r(),
                        chip_color.g(),
                        chip_color.b(),
                        140,
                    ),
                ),
                egui::StrokeKind::Inside,
            );
            painter.text(
                chip_rect.center(),
                egui::Align2::CENTER_CENTER,
                chip_text,
                egui::FontId::monospace(9.5),
                chip_color,
            );

            // Node name
            painter.text(
                row_rect.left_center() + egui::vec2(44.0, 0.0),
                egui::Align2::LEFT_CENTER,
                truncate_text(&label, 22),
                egui::FontId::proportional(12.5),
                if is_selected {
                    theme.colors.text
                } else {
                    theme.colors.text_secondary
                },
            );

            // Node type (dimmed, on the right)
            painter.text(
                row_rect.right_center() - egui::vec2(6.0, 0.0),
                egui::Align2::RIGHT_CENTER,
                node_type,
                egui::FontId::proportional(10.0),
                theme.colors.text_muted,
            );
        }

        if row_resp.clicked() {
            self.select_node(id);
        }
        row_resp.on_hover_text(self.node_label(id));
    }

    fn show_library_panel(&mut self, ui: &mut egui::Ui) {
        let theme = theme();
        widgets::section_label(ui, "GAMEPLAY PRESETS");
        widgets::inset_frame().show(ui, |ui| {
            for preset in GameplayPreset::ALL {
                if widgets::toolbar_button(ui, preset.label(), "Create preset node", true, false)
                    .clicked()
                {
                    self.create_preset_node(preset);
                }
            }
        });
        ui.add_space(theme.spacing.md);

        ui.horizontal(|ui| {
            let search_width =
                (ui.available_width() - theme.sizing.icon_button_size - 8.0).max(80.0);
            ui.add_sized(
                [search_width, 26.0],
                egui::TextEdit::singleline(&mut self.asset_filter_edit).hint_text("Search assets"),
            );
            if widgets::icon_button(ui, "R", "Refresh assets", true).clicked() {
                self.refresh_assets();
            }
        });

        ui.add_space(theme.spacing.sm);
        widgets::section_label(ui, "IMAGE ASSETS");
        if self.assets.is_empty() {
            widgets::inset_frame().show(ui, |ui| {
                ui.label(egui::RichText::new("No image assets").color(theme.colors.text_muted));
            });
            return;
        }

        let images = self.filtered_assets_by_text();
        widgets::inset_frame().show(ui, |ui| {
            if images.is_empty() {
                ui.label(egui::RichText::new("No image assets").color(theme.colors.text_muted));
                return;
            }

            egui::ScrollArea::vertical()
                .id_salt("library_asset_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for image in images.into_iter().take(32) {
                        let selected =
                            self.selected_asset_path.as_deref() == Some(image.asset_path.as_str());
                        let response = ui.add(
                            egui::Button::selectable(
                                selected,
                                truncate_text(&image.display_name, 24),
                            )
                            .fill(if selected {
                                theme.colors.control_active
                            } else {
                                theme.colors.panel_bg_alt
                            })
                            .stroke(egui::Stroke::new(
                                1.0,
                                if selected {
                                    theme.colors.accent
                                } else {
                                    theme.colors.border
                                },
                            ))
                            .corner_radius(theme.radius.sm)
                            .min_size(egui::vec2(ui.available_width(), 28.0)),
                        );
                        let clicked = response.clicked();
                        response.on_hover_text(image.asset_path.as_str());
                        if clicked {
                            self.choose_asset(image.asset_path, image.display_name.as_str(), true);
                        }
                    }
                });
        });
    }

    fn show_inspector(&mut self, root: &mut egui::Ui) {
        let theme = theme();
        egui::Panel::right("inspector_panel")
            .resizable(true)
            .default_size(theme.sizing.inspector_width)
            .min_size(280.0)
            .frame(widgets::panel_frame())
            .show_inside(root, |ui| {
                widgets::panel_header(ui, "Inspector", None);
                ui.add_space(theme.spacing.sm);

                let Some(entity) = self.selected else {
                    widgets::inset_frame().show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("No node selected").color(theme.colors.text_muted),
                        );
                    });
                    return;
                };

                let Some(node) = self.app.find_node(entity).cloned() else {
                    self.selected = None;
                    return;
                };

                self.show_selected_node_header(ui, entity, node.name.as_str());

                egui::ScrollArea::vertical()
                    .id_salt("inspector_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.show_node_inspector(ui, entity, node.name.as_str());
                        self.show_transform_inspector(ui, entity, node.transform);
                        self.show_tag_inspector(ui, entity);
                        self.show_sprite_inspector(ui, entity);
                        self.show_tilemap_inspector(ui, entity);
                        self.show_camera_inspector(ui, entity);
                        self.show_velocity_inspector(ui, entity);
                        self.show_collider_inspector(ui, entity);
                        self.show_player_controller_inspector(ui, entity);
                        self.show_camera_follow_inspector(ui, entity);
                        self.show_trigger_inspector(ui, entity);

                        ui.add_space(theme.spacing.md);
                        if widgets::add_component_button(ui).clicked() {
                            self.set_status(
                                "Expand a collapsed section below to add that component",
                            );
                        }
                        ui.add_space(theme.spacing.sm);
                    });
            });
    }

    fn show_selected_node_header(&mut self, ui: &mut egui::Ui, entity: EntityId, name: &str) {
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

    fn show_node_inspector(&mut self, ui: &mut egui::Ui, entity: EntityId, current_name: &str) {
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

    fn show_transform_inspector(
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

    fn create_preset_node(&mut self, preset: GameplayPreset) {
        match self.app.create_preset_node(preset) {
            Ok(entity) => {
                self.select_node(entity);
                self.set_success(format!("{} preset created", preset.label()));
            }
            Err(error) => self.set_error(format!("{error}")),
        }
    }

    fn show_tag_inspector(&mut self, ui: &mut egui::Ui, entity: EntityId) {
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

    fn show_sprite_inspector(&mut self, ui: &mut egui::Ui, entity: EntityId) {
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

    fn show_tilemap_inspector(&mut self, ui: &mut egui::Ui, entity: EntityId) {
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

    fn show_camera_inspector(&mut self, ui: &mut egui::Ui, entity: EntityId) {
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

    fn show_velocity_inspector(&mut self, ui: &mut egui::Ui, entity: EntityId) {
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

    fn show_collider_inspector(&mut self, ui: &mut egui::Ui, entity: EntityId) {
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

    fn show_player_controller_inspector(&mut self, ui: &mut egui::Ui, entity: EntityId) {
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

    fn show_camera_follow_inspector(&mut self, ui: &mut egui::Ui, entity: EntityId) {
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
            widgets::property_row(ui, "Action", |ui| {
                if widgets::toolbar_button(ui, "Apply", "Apply camera follow", true, false)
                    .clicked()
                {
                    match self.camera_follow_target_edit.trim().parse::<u64>() {
                        Ok(raw) => {
                            let mut follow = CameraFollowComponent::new(EntityId::from_raw(raw))
                                .with_smoothing(self.camera_follow_smoothing_edit);
                            if !self.camera_follow_enabled_edit {
                                follow = follow.disabled();
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

    fn show_trigger_inspector(&mut self, ui: &mut egui::Ui, entity: EntityId) {
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

    fn show_bottom_dock(&mut self, root: &mut egui::Ui) {
        let theme = theme();
        egui::Panel::bottom("bottom_dock")
            .resizable(true)
            .default_size(theme.sizing.bottom_dock_height)
            .min_size(145.0)
            .frame(widgets::panel_frame())
            .show_inside(root, |ui| {
                egui::Frame::new()
                    .fill(theme.colors.panel_header_bg)
                    .stroke(egui::Stroke::new(1.0, theme.colors.border))
                    .inner_margin(egui::Margin::symmetric(6, 4))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = theme.spacing.xs;
                            self.show_bottom_tab_button(
                                ui,
                                BottomDockTab::TilePalette,
                                "Tile Palette",
                            );
                            self.show_bottom_tab_button(ui, BottomDockTab::Assets, "Assets");
                            self.show_bottom_tab_button(ui, BottomDockTab::Output, "Output");
                            ui.add_enabled(
                                false,
                                egui::Button::selectable(false, "Debugger")
                                    .corner_radius(theme.radius.sm)
                                    .min_size(egui::vec2(80.0, 26.0))
                                    .fill(theme.colors.panel_bg_alt)
                                    .stroke(egui::Stroke::new(1.0, theme.colors.border)),
                            )
                            .on_hover_text("Debugger (coming soon)");

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    widgets::chip(
                                        ui,
                                        format!("{} assets", self.assets.images().len()).as_str(),
                                        StatusTone::Info,
                                    );
                                    widgets::status_badge(ui, &self.status, self.status_tone);
                                },
                            );
                        });
                    });
                ui.add_space(theme.spacing.sm);

                match self.bottom_tab {
                    BottomDockTab::TilePalette => self.show_tile_palette(ui),
                    BottomDockTab::Assets => self.show_asset_browser(ui),
                    BottomDockTab::Output => self.show_output_panel(ui),
                }
            });
    }

    fn show_bottom_tab_button(&mut self, ui: &mut egui::Ui, tab: BottomDockTab, label: &str) {
        if widgets::segment_button(ui, label, self.bottom_tab == tab).clicked() {
            self.bottom_tab = tab;
        }
    }

    fn show_tile_palette(&mut self, ui: &mut egui::Ui) {
        let theme = theme();
        widgets::inset_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                widgets::section_label(ui, "GROUND");
                widgets::chip(ui, self.active_tool.label(), StatusTone::Info);
                widgets::chip(
                    ui,
                    format!("Tile {}", self.selected_tile_index).as_str(),
                    StatusTone::Info,
                );
            });
            ui.add_space(theme.spacing.sm);
            ui.horizontal_wrapped(|ui| {
                for index in 0..16 {
                    let selected = self.selected_tile_index == index;
                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(theme.sizing.tile_button_size, theme.sizing.tile_button_size),
                        egui::Sense::click(),
                    );
                    let painter = ui.painter();
                    painter.rect_filled(rect, theme.radius.sm, tile_color(index));
                    painter.rect_stroke(
                        rect,
                        theme.radius.sm,
                        egui::Stroke::new(
                            if selected { 2.0 } else { 1.0 },
                            if selected {
                                theme.colors.accent
                            } else {
                                theme.colors.border_strong
                            },
                        ),
                        egui::StrokeKind::Inside,
                    );
                    painter.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        index.to_string(),
                        egui::FontId::monospace(11.0),
                        egui::Color32::from_rgba_unmultiplied(8, 12, 14, 180),
                    );
                    if response.clicked() {
                        self.selected_tile_index = index;
                        self.active_tool = EditorTool::TileBrush;
                        self.set_status(format!("Tile {index} selected"));
                    }
                }
            });
        });
    }

    fn show_asset_browser(&mut self, ui: &mut egui::Ui) {
        let theme = theme();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = theme.spacing.xs;
            widgets::toolbar_button(ui, "Import", "Import asset into project", false, false)
                .clicked();
            if widgets::icon_button(ui, "↺", "Refresh assets", true).clicked() {
                self.refresh_assets();
            }
            ui.add_sized(
                [220.0, 26.0],
                egui::TextEdit::singleline(&mut self.asset_filter_edit)
                    .hint_text("Search assets..."),
            );
            ui.separator();
            if widgets::segment_button(ui, "Images", self.asset_tab == AssetBrowserTab::Images)
                .clicked()
            {
                self.asset_tab = AssetBrowserTab::Images;
            }
            if widgets::segment_button(ui, "Broken", self.asset_tab == AssetBrowserTab::Broken)
                .clicked()
            {
                self.asset_tab = AssetBrowserTab::Broken;
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if widgets::toolbar_button(ui, "Clear", "Clear asset selection", true, false)
                    .clicked()
                {
                    self.selected_asset_path = None;
                    self.asset_filter_edit.clear();
                    self.set_status("Asset selection cleared");
                }
            });
        });
        ui.add_space(theme.spacing.sm);

        let images = self.filtered_assets_for_current_tab(ui.ctx());
        if images.is_empty() {
            widgets::inset_frame().show(ui, |ui| {
                ui.label(
                    egui::RichText::new("No matching image assets").color(theme.colors.text_muted),
                );
            });
            return;
        }

        egui::ScrollArea::horizontal()
            .id_salt("asset_browser_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for image in images {
                        let asset_path = image.asset_path.clone();
                        let display_name = image.display_name.clone();
                        let result = self.image_asset_card(ui, &image);
                        if result.clicked {
                            if let Some(error) = result.load_error {
                                self.selected_asset_path = Some(asset_path);
                                self.report_asset_error(error);
                            } else {
                                self.choose_asset(asset_path, display_name.as_str(), true);
                            }
                        }
                    }
                });
            });
    }

    fn show_output_panel(&mut self, ui: &mut egui::Ui) {
        widgets::inset_frame().show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("output_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for line in self.output.iter().rev() {
                        ui.colored_label(output_color(line), line);
                    }
                });
        });
    }

    fn show_logo(&mut self, ui: &mut egui::Ui) {
        let theme = theme();
        let (rect, response) = ui.allocate_exact_size(egui::vec2(38.0, 38.0), egui::Sense::hover());
        match self.textures.load(ui.ctx(), "logo.png") {
            TextureLookup::Loaded(texture) => {
                ui.painter().rect_filled(
                    rect.expand(2.0),
                    theme.radius.md,
                    theme.colors.panel_bg_alt,
                );
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
                camera: self.app.node_camera(node.id).copied(),
                tilemap: self.app.node_tilemap(node.id).cloned(),
                collider: self.app.node_collider(node.id).copied(),
                trigger: self.app.node_trigger(node.id).is_some(),
            })
            .collect();

        let mut clicked_id = None;
        let mut clicked_name = None;
        let mut paint_request = None;
        let mut asset_warning = None;
        let theme = theme();

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme.colors.viewport_bg))
            .show_inside(root, |ui| {
                let available = ui.available_size();
                let (rect, response) =
                    ui.allocate_exact_size(available, egui::Sense::click_and_drag());
                let painter = ui.painter_at(rect);
                let world_to_screen =
                    |position: Vec2| rect.center() + egui::vec2(position.x, -position.y);
                let screen_to_world = |position: egui::Pos2| {
                    Vec2::new(
                        position.x - rect.center().x,
                        -(position.y - rect.center().y),
                    )
                };

                painter.rect_filled(rect, 0.0, theme.colors.viewport_bg);
                draw_world_grid(&painter, rect, rect.center(), 32.0, theme.colors.grid_minor);
                draw_world_grid(
                    &painter,
                    rect,
                    rect.center(),
                    128.0,
                    theme.colors.grid_major,
                );
                draw_world_axes(&painter, rect, rect.center());

                let mut hit_rects = Vec::new();

                for item in &items {
                    draw_camera_frame(&painter, rect, &world_to_screen, item);
                }

                for item in &items {
                    if let Some(tilemap) = &item.tilemap {
                        let (tilemap_rect, warning) = self.draw_tilemap(
                            ui.ctx(),
                            &painter,
                            rect,
                            &world_to_screen,
                            item,
                            tilemap,
                        );
                        if asset_warning.is_none() {
                            asset_warning = warning;
                        }
                        hit_rects.push((item.id, tilemap_rect));
                    }
                }

                for item in &items {
                    let (hit_rect, warning) =
                        self.draw_node(ui.ctx(), &painter, &world_to_screen, item);
                    if asset_warning.is_none() {
                        asset_warning = warning;
                    }
                    hit_rects.push((item.id, hit_rect));
                }

                for item in &items {
                    draw_collider_overlay(&painter, &world_to_screen, item);
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

                if self.active_tool == EditorTool::Select {
                    if response.drag_started() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            if let Some((id, _)) =
                                hit_rects.iter().rev().find(|(_, hit)| hit.contains(pos))
                            {
                                self.select_node(*id);
                            }
                        }
                        if let Some(entity) = self.selected {
                            if let Some(before) = self.app.node_transform(entity) {
                                self.transform_drag = Some((entity, before));
                            }
                        }
                    }

                    if response.dragged() {
                        if let Some((entity, before)) = self.transform_drag {
                            let delta = response.drag_delta();
                            let mut transform = before;
                            transform.position = before.position + Vec2::new(delta.x, -delta.y);
                            if let Err(error) = self
                                .app
                                .execute_command(EditorCommand::move_node(entity, transform))
                            {
                                self.set_error(format!("{error}"));
                            }
                        }
                    }

                    if response.drag_stopped() {
                        if let Some((entity, before)) = self.transform_drag.take() {
                            if let Some(after) = self.app.node_transform(entity) {
                                self.app.record_move_node(entity, before, after);
                                self.set_status("Node moved");
                            }
                        }
                    }
                }

                self.show_viewport_overlays(ui, rect, asset_warning.as_deref());
            });

        if let Some(warning) = asset_warning {
            self.report_asset_error(warning);
        }

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

    fn show_viewport_overlays(&self, ui: &egui::Ui, rect: egui::Rect, asset_warning: Option<&str>) {
        let theme = theme();

        // Top-left: tool + tile + zoom + grid info bar
        egui::Area::new(egui::Id::new("viewport_status_overlay"))
            .order(egui::Order::Foreground)
            .fixed_pos(rect.left_top() + egui::vec2(10.0, 10.0))
            .show(ui.ctx(), |ui| {
                egui::Frame::new()
                    .fill(theme.colors.viewport_overlay)
                    .stroke(egui::Stroke::new(1.0, theme.colors.border))
                    .corner_radius(theme.radius.sm)
                    .inner_margin(egui::Margin::symmetric(8, 4))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = theme.spacing.sm;
                            widgets::chip(ui, self.active_tool.label(), StatusTone::Info)
                                .on_hover_text("Active tool — change in toolbar");
                            ui.separator();
                            widgets::chip(
                                ui,
                                format!("Tile {}", self.selected_tile_index).as_str(),
                                StatusTone::Info,
                            )
                            .on_hover_text("Selected tile index for painting");
                            ui.separator();
                            widgets::chip(ui, "100%", StatusTone::Info)
                                .on_hover_text("Viewport zoom");
                            widgets::chip(ui, "Grid 32", StatusTone::Info)
                                .on_hover_text("Grid step in pixels");
                        });
                    });
            });

        // Top-right: asset warning
        if let Some(warning) = asset_warning {
            let x = (rect.right() - 150.0).max(rect.left() + 12.0);
            egui::Area::new(egui::Id::new("viewport_asset_issue_overlay"))
                .order(egui::Order::Foreground)
                .fixed_pos(egui::pos2(x, rect.top() + 10.0))
                .show(ui.ctx(), |ui| {
                    egui::Frame::new()
                        .fill(theme.colors.viewport_overlay)
                        .stroke(egui::Stroke::new(1.0, theme.colors.border))
                        .corner_radius(theme.radius.sm)
                        .inner_margin(egui::Margin::symmetric(8, 4))
                        .show(ui, |ui| {
                            widgets::chip(ui, "! Asset issue", StatusTone::Error)
                                .on_hover_text(warning);
                        });
                });
        }

        // Bottom-left: world coordinates hint
        egui::Area::new(egui::Id::new("viewport_coords_overlay"))
            .order(egui::Order::Foreground)
            .fixed_pos(rect.left_bottom() + egui::vec2(10.0, -30.0))
            .show(ui.ctx(), |ui| {
                egui::Frame::new()
                    .fill(theme.colors.viewport_overlay)
                    .stroke(egui::Stroke::new(1.0, theme.colors.border))
                    .corner_radius(theme.radius.sm)
                    .inner_margin(egui::Margin::symmetric(8, 3))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("Pos (0, 0)  |  Layer 0")
                                .size(10.5)
                                .color(theme.colors.text_muted),
                        );
                    });
            });
    }

    fn draw_tilemap(
        &mut self,
        ctx: &egui::Context,
        painter: &egui::Painter,
        viewport_rect: egui::Rect,
        world_to_screen: &dyn Fn(Vec2) -> egui::Pos2,
        item: &NodeView,
        tilemap: &TilemapComponent,
    ) -> (egui::Rect, Option<String>) {
        let theme = theme();
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
                    texture_error =
                        Some(format!("Tileset asset failed for '{}': {error}", item.name));
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
                painter.rect_filled(
                    tile_rect.shrink(0.5),
                    0.0,
                    tile_color(visible.cell.tile_index),
                );
            }
        }

        let selected = self.selected == Some(item.id);
        painter.rect_stroke(
            map_rect,
            0.0,
            egui::Stroke::new(
                if selected { 2.0 } else { 1.0 },
                if selected {
                    theme.colors.accent
                } else {
                    theme.colors.border_strong
                },
            ),
            egui::StrokeKind::Inside,
        );
        if selected {
            draw_selection_handles(painter, map_rect);
        }
        if texture_error.is_some() {
            draw_corner_badge(painter, map_rect.left_top() + egui::vec2(10.0, 10.0), "!");
        }

        (map_rect, texture_error)
    }

    fn draw_node(
        &mut self,
        ctx: &egui::Context,
        painter: &egui::Painter,
        world_to_screen: &dyn Fn(Vec2) -> egui::Pos2,
        item: &NodeView,
    ) -> (egui::Rect, Option<String>) {
        let center = world_to_screen(item.transform.position);
        let is_selected = self.selected == Some(item.id);
        let color = self.node_color(item);
        let mut warning = None;

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
                    warning = Some(format!("Sprite asset failed for '{}': {error}", item.name));
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
                hit_rect.expand(5.0),
                theme().radius.sm,
                egui::Stroke::new(2.0, theme().colors.accent),
                egui::StrokeKind::Inside,
            );
            draw_selection_handles(painter, hit_rect.expand(5.0));
        }
        draw_node_label(
            painter,
            hit_rect.center_bottom() + egui::vec2(0.0, 7.0),
            &item.name,
        );
        (hit_rect, warning)
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

    fn image_asset_card(&mut self, ui: &mut egui::Ui, image: &ImageAsset) -> AssetCardResult {
        let theme = theme();
        let selected = self.selected_asset_path.as_deref() == Some(image.asset_path.as_str());
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(
                theme.sizing.asset_card_width,
                theme.sizing.asset_card_height,
            ),
            egui::Sense::click(),
        );
        let hovered = response.hovered();
        let clicked = response.clicked();
        let painter = ui.painter_at(rect);
        let fill = if selected {
            theme.colors.control_active
        } else if hovered {
            theme.colors.control_hover
        } else {
            theme.colors.panel_bg_alt
        };

        painter.rect_filled(rect, theme.radius.md, fill);
        painter.rect_stroke(
            rect,
            theme.radius.md,
            egui::Stroke::new(
                if selected { 2.0 } else { 1.0 },
                if selected {
                    theme.colors.accent
                } else {
                    theme.colors.border
                },
            ),
            egui::StrokeKind::Inside,
        );

        let thumbnail_rect = egui::Rect::from_min_size(
            rect.left_top() + egui::vec2(8.0, 8.0),
            egui::vec2(rect.width() - 16.0, 76.0),
        );
        painter.rect_filled(thumbnail_rect, theme.radius.sm, theme.colors.app_bg);

        let mut load_error = None;
        match self.textures.load(ui.ctx(), &image.asset_path) {
            TextureLookup::Loaded(texture) => {
                let image_rect = fit_rect(texture.size_vec2(), thumbnail_rect.shrink(6.0));
                painter.image(
                    texture.id(),
                    image_rect,
                    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }
            TextureLookup::Failed(error) => {
                draw_missing_texture_marker(&painter, thumbnail_rect.shrink(12.0), "!");
                load_error = Some(format!(
                    "Asset failed to load '{}': {error}",
                    image.asset_path
                ));
            }
            TextureLookup::Missing => {
                draw_missing_texture_marker(&painter, thumbnail_rect.shrink(12.0), "?");
            }
        }

        let name = truncate_text(&image.display_name, 18);
        painter.text(
            rect.left_top() + egui::vec2(10.0, 94.0),
            egui::Align2::LEFT_TOP,
            name,
            egui::FontId::proportional(12.0),
            theme.colors.text,
        );
        painter.text(
            rect.left_bottom() + egui::vec2(10.0, -22.0),
            egui::Align2::LEFT_TOP,
            truncate_text(&image.asset_path, 22),
            egui::FontId::monospace(10.0),
            theme.colors.text_muted,
        );

        let tooltip = if let Some(error) = load_error.as_deref() {
            format!("{}\n{error}", image.asset_path)
        } else {
            image.asset_path.clone()
        };
        response.on_hover_text(tooltip);

        AssetCardResult {
            clicked,
            load_error,
        }
    }

    fn apply_sprite(&mut self, entity: EntityId, sprite_path: String) {
        match self
            .app
            .execute_command_with_history(EditorCommand::attach_sprite(entity, sprite_path))
        {
            Ok(_) => {
                self.sync_selected_buffers();
                self.set_success("Sprite applied");
            }
            Err(error) => self.set_error(format!("{error}")),
        }
    }

    fn remove_component(
        &mut self,
        entity: EntityId,
        component: EditorComponentKind,
        success_message: &str,
    ) {
        match self
            .app
            .execute_command_with_history(EditorCommand::remove_component(entity, component))
        {
            Ok(_) => {
                self.sync_selected_buffers();
                self.set_success(success_message);
            }
            Err(error) => self.set_error(format!("{error}")),
        }
    }

    fn choose_asset(&mut self, asset_path: String, display_name: &str, apply_to_selected: bool) {
        self.selected_asset_path = Some(asset_path.clone());
        if apply_to_selected {
            if let Some(entity) = self.selected {
                self.sprite_edit = asset_path.clone();
                self.apply_sprite(entity, asset_path);
                return;
            }
        }
        self.set_status(format!("Asset selected: {display_name}"));
    }

    fn refresh_assets(&mut self) {
        let roots = asset_roots(self.app.asset_roots());
        self.textures.set_asset_roots(roots.clone());
        self.assets = ImageAssetCatalog::scan(&roots);
        self.set_success("Assets refreshed");
    }

    fn filtered_assets_by_text(&self) -> Vec<ImageAsset> {
        let filter = self.asset_filter_edit.trim().to_lowercase();
        self.assets
            .images()
            .iter()
            .filter(|image| {
                filter.is_empty()
                    || image.display_name.to_lowercase().contains(filter.as_str())
                    || image.asset_path.to_lowercase().contains(filter.as_str())
            })
            .cloned()
            .collect()
    }

    fn filtered_assets_for_current_tab(&mut self, ctx: &egui::Context) -> Vec<ImageAsset> {
        let images = self.filtered_assets_by_text();
        if self.asset_tab != AssetBrowserTab::Broken {
            return images;
        }

        images
            .into_iter()
            .filter(|image| {
                matches!(
                    self.textures.load(ctx, &image.asset_path),
                    TextureLookup::Failed(_)
                )
            })
            .collect()
    }

    fn node_label(&self, id: EntityId) -> String {
        let name = self
            .app
            .find_node(id)
            .map(|node| node.name.as_str())
            .unwrap_or("?");

        let (kind, _) = self.node_kind(id);
        format!("{kind} {name}")
    }

    fn node_kind(&self, id: EntityId) -> (&'static str, StatusTone) {
        if self.app.node_tilemap(id).is_some() {
            ("MAP", StatusTone::Info)
        } else if self.app.node_camera(id).is_some() {
            ("CAM", StatusTone::Warning)
        } else if self.app.node_player_controller(id).is_some() {
            ("PLY", StatusTone::Success)
        } else if self.app.node_trigger(id).is_some() {
            ("TRG", StatusTone::Warning)
        } else if self.app.node_sprite(id).is_some() {
            ("SPR", StatusTone::Success)
        } else if self.app.node_tag(id).is_some() {
            ("TAG", StatusTone::Info)
        } else {
            ("NOD", StatusTone::Info)
        }
    }

    fn node_type(&self, id: EntityId) -> &'static str {
        if self.app.node_tilemap(id).is_some() {
            "Tilemap"
        } else if self.app.node_camera(id).is_some() {
            "Camera2D"
        } else if self.app.node_player_controller(id).is_some() {
            "Player"
        } else if self.app.node_trigger(id).is_some() {
            "Trigger"
        } else if self.app.node_sprite(id).is_some() {
            "Sprite2D"
        } else {
            "Node2D"
        }
    }

    fn node_color(&self, item: &NodeView) -> egui::Color32 {
        let theme = theme();
        if self.selected == Some(item.id) {
            theme.colors.accent
        } else if item.camera.is_some() {
            theme.colors.camera
        } else if item.sprite_path.is_some() {
            theme.colors.success
        } else if item.tilemap.is_some() {
            theme.colors.accent_soft
        } else {
            theme.colors.warning
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct NodeView {
    id: EntityId,
    name: String,
    transform: Transform2D,
    sprite_path: Option<String>,
    camera: Option<Camera2DComponent>,
    tilemap: Option<TilemapComponent>,
    collider: Option<Collider2DComponent>,
    trigger: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditorTool {
    Select,
    TileBrush,
    EraseTile,
}

impl EditorTool {
    fn label(self) -> &'static str {
        match self {
            Self::Select => "Select",
            Self::TileBrush => "Tile Brush",
            Self::EraseTile => "Erase",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LeftPanelTab {
    Scene,
    Library,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BottomDockTab {
    TilePalette,
    Assets,
    Output,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AssetBrowserTab {
    Images,
    Broken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProjectDialog {
    NewProject,
    OpenProject,
    SaveAs,
    SaveBeforePlay,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AssetCardResult {
    clicked: bool,
    load_error: Option<String>,
}

fn drag_value(
    ui: &mut egui::Ui,
    value: &mut f32,
    speed: f64,
    changed: &mut bool,
    drag_started: &mut bool,
    drag_stopped: &mut bool,
) {
    let response = ui.add_sized(
        [theme().sizing.property_input_width, 22.0],
        egui::DragValue::new(value).speed(speed),
    );
    *changed |= response.changed();
    *drag_started |= response.drag_started();
    *drag_stopped |= response.drag_stopped();
}

fn draw_world_grid(
    painter: &egui::Painter,
    rect: egui::Rect,
    origin: egui::Pos2,
    step: f32,
    color: egui::Color32,
) {
    if step <= 0.0 {
        return;
    }

    let mut x = origin.x + ((rect.left() - origin.x) / step).floor() * step;
    while x <= rect.right() {
        painter.line_segment(
            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
            egui::Stroke::new(1.0, color),
        );
        x += step;
    }

    let mut y = origin.y + ((rect.top() - origin.y) / step).floor() * step;
    while y <= rect.bottom() {
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            egui::Stroke::new(1.0, color),
        );
        y += step;
    }
}

fn draw_world_axes(painter: &egui::Painter, rect: egui::Rect, origin: egui::Pos2) {
    let theme = theme();
    if rect.top() <= origin.y && origin.y <= rect.bottom() {
        painter.line_segment(
            [
                egui::pos2(rect.left(), origin.y),
                egui::pos2(rect.right(), origin.y),
            ],
            egui::Stroke::new(1.0, theme.colors.axis_x),
        );
    }
    if rect.left() <= origin.x && origin.x <= rect.right() {
        painter.line_segment(
            [
                egui::pos2(origin.x, rect.top()),
                egui::pos2(origin.x, rect.bottom()),
            ],
            egui::Stroke::new(1.0, theme.colors.axis_y),
        );
    }
}

fn draw_camera_frame(
    painter: &egui::Painter,
    viewport_rect: egui::Rect,
    world_to_screen: &dyn Fn(Vec2) -> egui::Pos2,
    item: &NodeView,
) {
    let Some(camera) = item.camera else {
        return;
    };
    let theme = theme();
    let zoom = camera.zoom.max(0.1);
    let size = egui::vec2(640.0 / zoom, 360.0 / zoom);
    let rect = egui::Rect::from_center_size(world_to_screen(item.transform.position), size);
    if !viewport_rect.intersects(rect) {
        return;
    }

    painter.rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(1.5, theme.colors.camera),
        egui::StrokeKind::Inside,
    );
    painter.text(
        rect.left_top() + egui::vec2(8.0, 6.0),
        egui::Align2::LEFT_TOP,
        "Camera2D",
        egui::FontId::monospace(11.0),
        theme.colors.camera,
    );
}

fn draw_collider_overlay(
    painter: &egui::Painter,
    world_to_screen: &dyn Fn(Vec2) -> egui::Pos2,
    item: &NodeView,
) {
    let Some(collider) = item.collider else {
        return;
    };

    let theme = theme();
    let aabb = collider.world_aabb(item.transform);
    let min = world_to_screen(aabb.min);
    let max = world_to_screen(aabb.max);
    let rect = egui::Rect::from_min_max(
        egui::pos2(min.x.min(max.x), min.y.min(max.y)),
        egui::pos2(min.x.max(max.x), min.y.max(max.y)),
    );
    let color = if collider.is_sensor || item.trigger {
        theme.colors.warning
    } else {
        theme.colors.success
    };
    painter.rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(1.5, color),
        egui::StrokeKind::Inside,
    );
}

fn draw_node_marker(painter: &egui::Painter, center: egui::Pos2, color: egui::Color32) {
    let theme = theme();
    let node_rect = egui::Rect::from_center_size(center, egui::vec2(34.0, 34.0));
    painter.rect_filled(node_rect, theme.radius.sm, color.gamma_multiply(0.42));
    painter.rect_stroke(
        node_rect,
        theme.radius.sm,
        egui::Stroke::new(1.5, color),
        egui::StrokeKind::Inside,
    );
    painter.circle_filled(center, 3.0, color);
}

fn draw_missing_texture_marker(painter: &egui::Painter, rect: egui::Rect, label: &str) {
    let theme = theme();
    painter.rect_filled(rect, theme.radius.sm, theme.colors.error_bg);
    painter.rect_stroke(
        rect,
        theme.radius.sm,
        egui::Stroke::new(1.5, theme.colors.error),
        egui::StrokeKind::Inside,
    );
    painter.line_segment(
        [rect.left_top(), rect.right_bottom()],
        egui::Stroke::new(1.0, theme.colors.error),
    );
    painter.line_segment(
        [rect.right_top(), rect.left_bottom()],
        egui::Stroke::new(1.0, theme.colors.error),
    );
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(18.0),
        egui::Color32::from_rgb(255, 220, 220),
    );
}

fn draw_selection_handles(painter: &egui::Painter, rect: egui::Rect) {
    let theme = theme();
    for corner in [
        rect.left_top(),
        rect.right_top(),
        rect.left_bottom(),
        rect.right_bottom(),
    ] {
        let handle = egui::Rect::from_center_size(corner, egui::vec2(6.0, 6.0));
        painter.rect_filled(handle, theme.radius.xs, theme.colors.app_bg);
        painter.rect_stroke(
            handle,
            theme.radius.xs,
            egui::Stroke::new(1.5, theme.colors.accent),
            egui::StrokeKind::Inside,
        );
    }
}

fn draw_corner_badge(painter: &egui::Painter, position: egui::Pos2, label: &str) {
    let theme = theme();
    let rect = egui::Rect::from_center_size(position, egui::vec2(20.0, 20.0));
    painter.rect_filled(rect, theme.radius.sm, theme.colors.error_bg);
    painter.rect_stroke(
        rect,
        theme.radius.sm,
        egui::Stroke::new(1.0, theme.colors.error),
        egui::StrokeKind::Inside,
    );
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(13.0),
        theme.colors.error,
    );
}

fn draw_node_label(painter: &egui::Painter, position: egui::Pos2, label: &str) {
    let theme = theme();
    let label = truncate_text(label, 24);
    let width = (label.chars().count() as f32 * 7.0 + 14.0).clamp(36.0, 190.0);
    let rect =
        egui::Rect::from_center_size(position + egui::vec2(0.0, 9.0), egui::vec2(width, 18.0));
    painter.rect_filled(rect, theme.radius.sm, theme.colors.viewport_overlay);
    painter.rect_stroke(
        rect,
        theme.radius.sm,
        egui::Stroke::new(1.0, theme.colors.border),
        egui::StrokeKind::Inside,
    );
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::monospace(11.0),
        theme.colors.text,
    );
}

fn draw_fallback_logo(painter: &egui::Painter, rect: egui::Rect) {
    let theme = theme();
    painter.rect_filled(rect, theme.radius.md, theme.colors.accent);
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "C",
        egui::FontId::proportional(18.0),
        egui::Color32::from_rgb(10, 18, 20),
    );
}

fn fit_rect(image_size: egui::Vec2, container: egui::Rect) -> egui::Rect {
    if image_size.x <= 0.0 || image_size.y <= 0.0 {
        return container;
    }

    let scale = (container.width() / image_size.x).min(container.height() / image_size.y);
    egui::Rect::from_center_size(container.center(), image_size * scale)
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

fn asset_roots(project_roots: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut roots = project_roots;
    roots.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"));
    if let Ok(current_dir) = std::env::current_dir() {
        roots.push(current_dir.join("assets"));
        roots.push(current_dir.join("apps/crab2d-editor/assets"));
    }
    roots.dedup();
    roots
}

fn default_project_path(project_name: &str) -> String {
    let folder = sanitize_project_folder(project_name);
    PathBuf::from("projects").join(folder).display().to_string()
}

fn default_project_file_path(project_name: &str) -> String {
    PathBuf::from(default_project_path(project_name))
        .join("project.crab2d.json")
        .display()
        .to_string()
}

fn sanitize_project_folder(project_name: &str) -> String {
    let sanitized = project_name
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric() || *character == '_' || *character == '-'
        })
        .collect::<String>();
    if sanitized.is_empty() {
        "UntitledProject".to_owned()
    } else {
        sanitized
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_owned();
    }

    let take = max_chars.saturating_sub(3);
    let mut compact = text.chars().take(take).collect::<String>();
    compact.push_str("...");
    compact
}

fn format_solid_tiles(solid_tiles: &BTreeSet<u32>) -> String {
    solid_tiles
        .iter()
        .map(|tile| tile.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn parse_solid_tiles(input: &str) -> Result<BTreeSet<u32>, String> {
    let mut tiles = BTreeSet::new();
    for part in input
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        let tile = part
            .parse::<u32>()
            .map_err(|_| format!("Invalid tile index: {part}"))?;
        tiles.insert(tile);
    }
    Ok(tiles)
}

fn output_level(tone: StatusTone) -> &'static str {
    match tone {
        StatusTone::Info => "INFO",
        StatusTone::Success => "OK",
        StatusTone::Warning => "WARN",
        StatusTone::Error => "ERROR",
    }
}

fn output_color(line: &str) -> egui::Color32 {
    let theme = theme();
    if line.contains("[ERROR]") {
        theme.colors.error
    } else if line.contains("[WARN]") {
        theme.colors.warning
    } else if line.contains("[OK]") {
        theme.colors.success
    } else {
        theme.colors.text_secondary
    }
}

fn trim_output(output: &mut Vec<String>) {
    const MAX_LINES: usize = 200;
    if output.len() > MAX_LINES {
        let drain_count = output.len() - MAX_LINES;
        output.drain(0..drain_count);
    }
}
