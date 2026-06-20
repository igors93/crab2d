use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

use crab2d_core::{Engine, EngineConfig, FrameStep, ProjectDocument};
use crab2d_platform::{InputState, KeyCode, PlatformEvent};
use crab2d_render::{RenderItem, RenderList, SpriteRenderCommand, TilemapRenderCommand};
use crab2d_scene::{
    CameraFollowComponent, Collider2DComponent, PlayerControllerComponent, Scene, Vec2,
    Velocity2DComponent,
};
use eframe::egui;

fn main() {
    if let Err(error) = run() {
        eprintln!("Crab2D runtime failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let project_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(ProjectDocument::DEFAULT_FILE_NAME));
    let app = RuntimeApp::load(&project_path)?;

    eframe::run_native(
        "Crab2D Runtime",
        eframe::NativeOptions {
            renderer: eframe::Renderer::Glow,
            viewport: egui::ViewportBuilder::default()
                .with_title("Crab2D Runtime")
                .with_inner_size([960.0, 540.0])
                .with_min_inner_size([640.0, 360.0]),
            ..Default::default()
        },
        Box::new(|cc| {
            let mut app = app;
            app.renderer.configure(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
    .map_err(|error| error.to_string())
}

struct RuntimeApp {
    engine: Engine,
    input: InputState,
    previous_keys: BTreeSet<KeyCode>,
    last_frame: Instant,
    last_step: FrameStep,
    renderer: EguiRuntimeRenderer,
}

impl RuntimeApp {
    fn load(project_path: &Path) -> Result<Self, String> {
        let mut engine = Engine::new(EngineConfig::new("Crab2D Runtime"));
        engine
            .load_project_document(project_path)
            .map_err(|error| error.to_string())?;
        ensure_runtime_defaults(&mut engine);

        Ok(Self {
            engine,
            input: InputState::default(),
            previous_keys: BTreeSet::new(),
            last_frame: Instant::now(),
            last_step: FrameStep::empty(0.0),
            renderer: EguiRuntimeRenderer::new(asset_roots(project_path)),
        })
    }

    fn collect_input(&mut self, ctx: &egui::Context) {
        self.input.begin_frame();
        let current_keys = runtime_keys()
            .into_iter()
            .filter_map(|(egui_key, key_code)| {
                ctx.input(|input| input.key_down(egui_key))
                    .then_some(key_code)
            })
            .collect::<BTreeSet<_>>();

        for key in current_keys.difference(&self.previous_keys).copied() {
            self.input.apply_event(PlatformEvent::KeyPressed(key));
        }
        for key in self.previous_keys.difference(&current_keys).copied() {
            self.input.apply_event(PlatformEvent::KeyReleased(key));
        }
        if let Some(position) = ctx.input(|input| input.pointer.hover_pos()) {
            self.input.apply_event(PlatformEvent::CursorMoved {
                x: position.x,
                y: position.y,
            });
        }
        self.previous_keys = current_keys;
    }
}

impl eframe::App for RuntimeApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        let delta_seconds = (now - self.last_frame).as_secs_f32().clamp(0.0, 1.0 / 20.0);
        self.last_frame = now;

        let ctx = ui.ctx().clone();
        self.collect_input(&ctx);
        if self.input.was_key_pressed(KeyCode::Escape) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        match self.engine.tick_with_input(delta_seconds, &self.input) {
            Ok(step) => self.last_step = step,
            Err(error) => eprintln!("Runtime tick failed: {error}"),
        }

        self.renderer
            .draw(ui, &self.engine.active_scene, &self.last_step);
        ctx.request_repaint();
    }
}

fn runtime_keys() -> [(egui::Key, KeyCode); 9] {
    [
        (egui::Key::W, KeyCode::Character('w')),
        (egui::Key::A, KeyCode::Character('a')),
        (egui::Key::S, KeyCode::Character('s')),
        (egui::Key::D, KeyCode::Character('d')),
        (egui::Key::ArrowUp, KeyCode::ArrowUp),
        (egui::Key::ArrowDown, KeyCode::ArrowDown),
        (egui::Key::ArrowLeft, KeyCode::ArrowLeft),
        (egui::Key::ArrowRight, KeyCode::ArrowRight),
        (egui::Key::Escape, KeyCode::Escape),
    ]
}

fn ensure_runtime_defaults(engine: &mut Engine) {
    let player = engine
        .active_scene
        .find_node_by_tag("player")
        .or_else(|| engine.active_scene.find_node_by_name("Player"))
        .map(|node| node.id);

    if let Some(player) = player {
        if engine.active_scene.velocity(player).is_none() {
            let _ = engine
                .active_scene
                .add_velocity(player, Velocity2DComponent::default());
        }
        if engine.active_scene.player_controller(player).is_none() {
            let _ = engine
                .active_scene
                .add_player_controller(player, PlayerControllerComponent::default());
        }
        if engine.active_scene.collider(player).is_none() {
            let _ = engine.active_scene.add_collider(
                player,
                Collider2DComponent::rectangle(Vec2::new(24.0, 24.0)),
            );
        }

        if let Some(camera) = engine
            .active_scene
            .nodes()
            .iter()
            .find(|node| engine.active_scene.camera(node.id).is_some())
            .map(|node| node.id)
        {
            if engine.active_scene.camera_follow(camera).is_none() {
                let _ = engine
                    .active_scene
                    .add_camera_follow(camera, CameraFollowComponent::new(player));
            }
        }
    }
}

struct EguiRuntimeRenderer {
    asset_roots: Vec<PathBuf>,
    textures: BTreeMap<String, egui::TextureHandle>,
}

impl EguiRuntimeRenderer {
    fn new(asset_roots: Vec<PathBuf>) -> Self {
        Self {
            asset_roots,
            textures: BTreeMap::new(),
        }
    }

    fn configure(&mut self, ctx: &egui::Context) {
        let mut style = (*ctx.global_style()).clone();
        style.visuals = egui::Visuals::dark();
        style.visuals.panel_fill = egui::Color32::from_rgb(10, 12, 14);
        ctx.set_global_style(style);
    }

    fn draw(&mut self, ui: &mut egui::Ui, scene: &Scene, frame_step: &FrameStep) {
        let available = ui.available_size();
        let (rect, _) = ui.allocate_exact_size(available, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let render_list = RenderList::from_scene(scene);
        let clear_color = render_list
            .camera
            .map(|camera| rgba_f32_to_color(camera.clear_color))
            .unwrap_or_else(|| egui::Color32::from_rgb(16, 18, 20));
        painter.rect_filled(rect, 0.0, clear_color);

        for item in &render_list.items {
            match item {
                RenderItem::Tilemap(tilemap) => {
                    self.draw_tilemap(ui.ctx(), &painter, rect, &render_list, tilemap)
                }
                RenderItem::Sprite(sprite) => {
                    self.draw_sprite(ui.ctx(), &painter, rect, &render_list, sprite)
                }
            }
        }

        self.draw_overlay(&painter, rect, frame_step);
    }

    fn draw_tilemap(
        &mut self,
        ctx: &egui::Context,
        painter: &egui::Painter,
        viewport: egui::Rect,
        render_list: &RenderList,
        tilemap: &TilemapRenderCommand,
    ) {
        let texture = tilemap
            .tileset_path
            .as_deref()
            .and_then(|path| self.load_texture(ctx, path).map(|texture| texture.id()));

        for tile in &tilemap.tiles {
            let world_center = Vec2::new(
                tilemap.transform.position.x
                    + tile.x as f32 * tilemap.tile_width as f32
                    + tilemap.tile_width as f32 / 2.0,
                tilemap.transform.position.y
                    + tile.y as f32 * tilemap.tile_height as f32
                    + tilemap.tile_height as f32 / 2.0,
            );
            let tile_rect = world_rect(
                viewport,
                render_list,
                world_center,
                Vec2::new(tilemap.tile_width as f32, tilemap.tile_height as f32),
            );
            if !viewport.intersects(tile_rect) {
                continue;
            }

            if let (Some(texture), Some(columns), Some(rows)) =
                (texture, tilemap.tileset_columns, tilemap.tileset_rows)
            {
                painter.image(
                    texture,
                    tile_rect,
                    tile_uv(tile.tile_index, columns, rows),
                    rgba_u8_to_color(tile.tint_rgba),
                );
            } else {
                painter.rect_filled(tile_rect, 0.0, fallback_tile_color(tile.tile_index));
            }
        }
    }

    fn draw_sprite(
        &mut self,
        ctx: &egui::Context,
        painter: &egui::Painter,
        viewport: egui::Rect,
        render_list: &RenderList,
        sprite: &SpriteRenderCommand,
    ) {
        let Some(texture) = self.load_texture(ctx, &sprite.sprite_path) else {
            let rect = world_rect(
                viewport,
                render_list,
                sprite.transform.position,
                Vec2::new(24.0, 24.0),
            );
            painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(220, 80, 95));
            return;
        };

        let size = texture.size_vec2();
        let sprite_size = Vec2::new(
            size.x * sprite.transform.scale.x.abs().max(0.1),
            size.y * sprite.transform.scale.y.abs().max(0.1),
        );
        let rect = world_rect(
            viewport,
            render_list,
            sprite.transform.position,
            sprite_size,
        );
        painter.image(
            texture.id(),
            rect,
            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    }

    fn draw_overlay(&self, painter: &egui::Painter, rect: egui::Rect, frame_step: &FrameStep) {
        let text = format!(
            "Crab2D Runtime  |  collisions: {}  triggers: {}",
            frame_step.solid_collisions.len(),
            frame_step.triggers.len()
        );
        let bg = egui::Rect::from_min_size(
            rect.left_top() + egui::vec2(12.0, 12.0),
            egui::vec2(330.0, 26.0),
        );
        painter.rect_filled(bg, 5.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 170));
        painter.text(
            bg.left_center() + egui::vec2(10.0, 0.0),
            egui::Align2::LEFT_CENTER,
            text,
            egui::FontId::monospace(12.0),
            egui::Color32::from_rgb(230, 238, 240),
        );
    }

    fn load_texture(
        &mut self,
        ctx: &egui::Context,
        asset_path: &str,
    ) -> Option<&egui::TextureHandle> {
        let normalized = normalize_asset_path(asset_path);
        if !self.textures.contains_key(&normalized) {
            let path = resolve_path(&self.asset_roots, &normalized);
            let image = image::open(path).ok()?.to_rgba8();
            let size = [image.width() as usize, image.height() as usize];
            let pixels = image.into_raw();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
            let texture = ctx.load_texture(
                format!("runtime_asset:{normalized}"),
                color_image,
                egui::TextureOptions::NEAREST,
            );
            self.textures.insert(normalized.clone(), texture);
        }
        self.textures.get(&normalized)
    }
}

fn world_rect(
    viewport: egui::Rect,
    render_list: &RenderList,
    world_center: Vec2,
    world_size: Vec2,
) -> egui::Rect {
    let camera_position = render_list
        .camera
        .map(|camera| camera.transform.position)
        .unwrap_or(Vec2::ZERO);
    let zoom = render_list.camera.map(|camera| camera.zoom).unwrap_or(1.0);
    let relative = world_center - camera_position;
    let screen_center = viewport.center() + egui::vec2(relative.x * zoom, -relative.y * zoom);
    egui::Rect::from_center_size(
        screen_center,
        egui::vec2(world_size.x * zoom, world_size.y * zoom),
    )
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

fn asset_roots(project_path: &Path) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(parent) = project_path.parent() {
        roots.push(parent.to_path_buf());
        roots.push(parent.join("assets"));
    }
    if let Ok(current_dir) = std::env::current_dir() {
        roots.push(current_dir.join("assets"));
        roots.push(current_dir.join("apps/crab2d-editor/assets"));
        roots.push(current_dir.join("apps/crab2d-runtime/assets"));
    }
    roots
}

fn resolve_path(asset_roots: &[PathBuf], normalized_path: &str) -> PathBuf {
    let path = Path::new(normalized_path);
    if path.is_absolute() {
        return path.to_path_buf();
    }
    for root in asset_roots {
        let candidate = root.join(path);
        if candidate.exists() {
            return candidate;
        }
    }
    path.to_path_buf()
}

fn normalize_asset_path(path: &str) -> String {
    path.trim()
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_owned()
}

fn rgba_f32_to_color(rgba: [f32; 4]) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(
        (rgba[0].clamp(0.0, 1.0) * 255.0) as u8,
        (rgba[1].clamp(0.0, 1.0) * 255.0) as u8,
        (rgba[2].clamp(0.0, 1.0) * 255.0) as u8,
        (rgba[3].clamp(0.0, 1.0) * 255.0) as u8,
    )
}

fn rgba_u8_to_color(rgba: [u8; 4]) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3])
}

fn fallback_tile_color(tile_index: u32) -> egui::Color32 {
    match tile_index % 8 {
        0 => egui::Color32::from_rgb(82, 148, 74),
        1 => egui::Color32::from_rgb(116, 174, 79),
        2 => egui::Color32::from_rgb(169, 142, 88),
        3 => egui::Color32::from_rgb(91, 105, 86),
        4 => egui::Color32::from_rgb(57, 119, 169),
        5 => egui::Color32::from_rgb(142, 97, 174),
        6 => egui::Color32::from_rgb(201, 126, 62),
        _ => egui::Color32::from_rgb(169, 194, 204),
    }
}
