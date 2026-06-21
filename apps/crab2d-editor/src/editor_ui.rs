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

mod bottom_dock;
mod dialogs;
mod drawing;
mod inspector;
mod inspector_add;
mod inspector_basic;
mod inspector_components;
mod inspector_future;
mod inspector_gameplay;
mod node_meta;
mod scene_editing;
mod scene_panel;
mod top_bar;
mod types;
mod utils;
mod viewport;
mod viewport_overlay;
mod viewport_render;
mod workspace;

use self::drawing::*;
use self::types::*;
use self::utils::*;

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
    camera_follow_lock_x_edit: bool,
    camera_follow_lock_y_edit: bool,
    camera_follow_dead_zone_edit: f32,
    trigger_name_edit: String,
    trigger_once_edit: bool,
    tile_collision_edit: String,
    // Behavior
    behavior_script_edit: String,
    behavior_enabled_edit: bool,
    // Audio
    audio_clip_edit: String,
    audio_volume_edit: f32,
    audio_looping_edit: bool,
    // Animation
    animation_spritesheet_edit: String,
    // UiLabel
    ui_label_text_edit: String,
    ui_label_font_size_edit: f32,
    // Particle
    particle_texture_edit: String,
    particle_emit_rate_edit: f32,
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
    asset_category: AssetCategory,
    selected_asset_path: Option<String>,
    asset_drag: Option<AssetPlacementDrag>,
    viewport_context_world: Option<Vec2>,
    scene_panel_slot: DockSlot,
    inspector_slot: DockSlot,
    asset_dock_slot: DockSlot,
    scene_panel_visible: bool,
    inspector_visible: bool,
    asset_dock_visible: bool,
    asset_dock_collapsed: bool,
    show_grid: bool,
    snap_enabled: bool,
    active_layer: String,
    inspector_filter_edit: String,
    status: String,
    status_tone: StatusTone,
    output: Vec<String>,
    last_asset_error: Option<String>,
    transform_drag: Option<(EntityId, Transform2D)>,
    viewport_drag: Option<ViewportDrag>,
    ui_zoom: Option<f32>,
    /// 2D viewport pan offset in world units
    viewport_pan: egui::Vec2,
    /// 2D viewport zoom (pixels per world unit, default 1.0)
    viewport_zoom: f32,
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
            camera_follow_lock_x_edit: false,
            camera_follow_lock_y_edit: false,
            camera_follow_dead_zone_edit: 0.0,
            trigger_name_edit: "trigger".to_owned(),
            trigger_once_edit: false,
            tile_collision_edit: String::new(),
            behavior_script_edit: "scripts/entity.rhai".to_owned(),
            behavior_enabled_edit: true,
            audio_clip_edit: "audio/sound.wav".to_owned(),
            audio_volume_edit: 1.0,
            audio_looping_edit: false,
            animation_spritesheet_edit: "sprites/sheet.png".to_owned(),
            ui_label_text_edit: "Label".to_owned(),
            ui_label_font_size_edit: 16.0,
            particle_texture_edit: "sprites/particle.png".to_owned(),
            particle_emit_rate_edit: 20.0,
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
            bottom_tab: BottomDockTab::Assets,
            asset_tab: AssetBrowserTab::Images,
            asset_category: AssetCategory::All,
            selected_asset_path: None,
            asset_drag: None,
            viewport_context_world: None,
            scene_panel_slot: DockSlot::Left,
            inspector_slot: DockSlot::Right,
            asset_dock_slot: DockSlot::Bottom,
            scene_panel_visible: true,
            inspector_visible: true,
            asset_dock_visible: true,
            asset_dock_collapsed: false,
            show_grid: true,
            snap_enabled: true,
            active_layer: "Ground".to_owned(),
            inspector_filter_edit: String::new(),
            status: "Ready".to_owned(),
            status_tone: StatusTone::Info,
            output: vec![
                "[INFO] Crab2D editor started".to_owned(),
                "[INFO] Starter scene loaded".to_owned(),
            ],
            last_asset_error: None,
            transform_drag: None,
            viewport_drag: None,
            ui_zoom: None,
            viewport_pan: egui::Vec2::ZERO,
            viewport_zoom: 1.0,
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
            self.camera_follow_lock_x_edit = false;
            self.camera_follow_lock_y_edit = false;
            self.camera_follow_dead_zone_edit = 0.0;
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
        self.camera_follow_lock_x_edit = follow.lock_x;
        self.camera_follow_lock_y_edit = follow.lock_y;
        self.camera_follow_dead_zone_edit = follow.dead_zone;

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

    fn reload_scene_from_disk(&mut self) {
        match self.app.reload_active_scene() {
            Ok(()) => {
                self.select_default_node();
                self.sync_selected_buffers();
                self.set_success("Scene reloaded from disk");
            }
            Err(error) => self.set_error(format!("Reload failed: {error}")),
        }
    }

    fn create_node(&mut self) {
        self.create_node_named("Node");
    }

    fn create_node_named(&mut self, name: &str) {
        match self
            .app
            .execute_command_with_history(EditorCommand::create_node(name))
        {
            Ok(EditorCommandResult::CreatedNode(id)) => {
                self.select_node(id);
                self.set_success("Node created");
            }
            Ok(EditorCommandResult::None) => {}
            Err(error) => self.set_error(format!("{error}")),
        }
    }

    fn create_camera_node(&mut self) {
        match self
            .app
            .execute_command_with_history(EditorCommand::create_camera("Camera2D"))
        {
            Ok(EditorCommandResult::CreatedNode(id)) => {
                self.select_node(id);
                self.set_success("Camera created");
            }
            Ok(EditorCommandResult::None) => {}
            Err(error) => self.set_error(format!("{error}")),
        }
    }

    fn create_world_text_node(&mut self) {
        match self
            .app
            .execute_command_with_history(EditorCommand::create_world_text_node(
                "WorldText",
                "New text",
            )) {
            Ok(EditorCommandResult::CreatedNode(id)) => {
                self.select_node(id);
                self.set_success("World text created");
            }
            Ok(EditorCommandResult::None) => {}
            Err(error) => self.set_error(format!("{error}")),
        }
    }
}

impl eframe::App for Crab2DEditorUi {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        if let Some(target) = crate::editor_theme::adaptive_zoom(&ctx) {
            let should_update = self
                .ui_zoom
                .map(|current| (current - target).abs() > 0.015)
                .unwrap_or(true);
            if should_update {
                ctx.set_zoom_factor(target);
                self.ui_zoom = Some(target);
            }
        }

        ctx.send_viewport_cmd(egui::ViewportCommand::Title(
            self.app.project_session().display_title(),
        ));
        self.handle_shortcuts(&ctx);
        self.show_top_bar(ui);
        self.show_top_bar_gap(ui);
        self.show_scene_tabs(ui);
        self.show_status_bar(ui);
        self.show_workspace(ui);
        self.show_project_dialogs(&ctx);
    }
}
