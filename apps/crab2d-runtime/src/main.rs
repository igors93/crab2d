use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

use crab2d_core::{
    animation_system,
    audio_system::AudioSystem,
    particle_system::ParticleSystem,
    save_system::GameSave,
    scene_manager::SceneManager,
    script_runtime::{ScriptContext, ScriptOutput, ScriptRuntime},
    Engine, EngineConfig, FrameStep, ProjectDocument,
};
use crab2d_platform::{InputState, KeyCode, MouseButton, PlatformEvent};
use crab2d_render::{RenderItem, RenderList, SpriteRenderCommand, TilemapRenderCommand};
use crab2d_scene::{
    CameraFollowComponent, Collider2DComponent, EntityId, PlayerControllerComponent, Scene,
    UiAnchor, Vec2, Velocity2DComponent,
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
    script_runtime: ScriptRuntime,
    scripts_started: BTreeSet<(u64, String)>,
    /// Maps script path → last modified time for hot-reload
    script_mtimes: BTreeMap<String, std::time::SystemTime>,
    audio_system: AudioSystem,
    particle_system: ParticleSystem,
    scene_manager: SceneManager,
    #[allow(dead_code)]
    game_save: GameSave,
    asset_roots: Vec<PathBuf>,
    debug_overlay: bool,
    fps_history: std::collections::VecDeque<f32>,
}

impl RuntimeApp {
    fn load(project_path: &Path) -> Result<Self, String> {
        let mut engine = Engine::new(EngineConfig::new("Crab2D Runtime"));
        engine
            .load_project_document(project_path)
            .map_err(|error| error.to_string())?;
        ensure_runtime_defaults(&mut engine);

        let roots = asset_roots(project_path);
        let save_dir = project_path
            .parent()
            .map(|p| p.join("saves"))
            .unwrap_or_else(|| PathBuf::from("saves"));

        Ok(Self {
            engine,
            input: InputState::default(),
            previous_keys: BTreeSet::new(),
            last_frame: Instant::now(),
            last_step: FrameStep::empty(0.0),
            renderer: EguiRuntimeRenderer::new(roots.clone()),
            script_runtime: ScriptRuntime::new(),
            scripts_started: BTreeSet::new(),
            script_mtimes: BTreeMap::new(),
            audio_system: AudioSystem::new(roots.clone()),
            particle_system: ParticleSystem::new(),
            scene_manager: SceneManager::new(roots.clone()),
            game_save: GameSave::new(save_dir),
            asset_roots: roots,
            debug_overlay: false,
            fps_history: std::collections::VecDeque::with_capacity(60),
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

        // Mouse buttons
        let mouse_buttons = [
            (egui::PointerButton::Primary, MouseButton::Left),
            (egui::PointerButton::Secondary, MouseButton::Right),
            (egui::PointerButton::Middle, MouseButton::Middle),
        ];
        for (egui_btn, btn) in mouse_buttons {
            if ctx.input(|i| i.pointer.button_pressed(egui_btn)) {
                self.input
                    .apply_event(PlatformEvent::MouseButtonPressed(btn));
            }
            if ctx.input(|i| i.pointer.button_released(egui_btn)) {
                self.input
                    .apply_event(PlatformEvent::MouseButtonReleased(btn));
            }
        }
        let scroll = ctx.input(|i| i.smooth_scroll_delta);
        if scroll.x != 0.0 || scroll.y != 0.0 {
            self.input.apply_event(PlatformEvent::MouseScrolled {
                delta_x: scroll.x,
                delta_y: scroll.y,
            });
        }

        self.previous_keys = current_keys;
    }

    fn run_script_frame(&mut self, delta: f32) {
        let behavior_entities: Vec<(EntityId, String, bool)> = self
            .engine
            .active_scene
            .behaviors()
            .map(|(e, b)| (e, b.script_path.clone(), b.enabled))
            .collect();

        for (_, path, _) in &behavior_entities {
            let full = resolve_path(&self.asset_roots, path);
            let mtime = std::fs::metadata(&full).and_then(|m| m.modified()).ok();
            let needs_load = if let Some(mtime) = mtime {
                let changed = self
                    .script_mtimes
                    .get(path.as_str())
                    .map(|&prev| prev != mtime)
                    .unwrap_or(true);
                if changed {
                    self.script_mtimes.insert(path.clone(), mtime);
                }
                changed || !self.script_runtime.is_loaded(path)
            } else {
                !self.script_runtime.is_loaded(path)
            };
            if needs_load {
                match std::fs::read_to_string(&full) {
                    Ok(source) => {
                        if let Err(e) = self.script_runtime.load_script(path, &source) {
                            eprintln!("Script '{path}' failed to load: {e}");
                        } else {
                            // Reset started state so on_start fires again after reload
                            self.scripts_started.retain(|(_, p)| p != path);
                        }
                    }
                    Err(e) => eprintln!("Script '{path}' not found at {}: {e}", full.display()),
                }
            }
        }

        for (entity, path, enabled) in &behavior_entities {
            if !enabled {
                continue;
            }
            let key = (entity.raw(), path.clone());
            if !self.scripts_started.contains(&key) {
                let ctx = script_context(&self.engine, *entity, &self.input);
                match self.script_runtime.call_on_start(path, &ctx) {
                    Ok(output) => self.apply_script_output(*entity, output),
                    Err(e) => eprintln!("[Script ERROR] {e}"),
                }
                self.scripts_started.insert(key);
            }
        }

        for (entity, path, enabled) in &behavior_entities {
            if !enabled {
                continue;
            }
            let ctx = script_context(&self.engine, *entity, &self.input);
            match self.script_runtime.call_on_update(path, &ctx, delta) {
                Ok(output) => self.apply_script_output(*entity, output),
                Err(e) => eprintln!("[Script ERROR] {e}"),
            }
        }

        let triggers = self.last_step.triggers.clone();
        for trigger in triggers {
            for entity in [trigger.trigger_entity, trigger.activator] {
                let Some(behavior) = self.engine.active_scene.behavior(entity).cloned() else {
                    continue;
                };
                if !behavior.enabled {
                    continue;
                }
                let ctx = script_context(&self.engine, entity, &self.input);
                match self.script_runtime.call_on_trigger(
                    &behavior.script_path,
                    &ctx,
                    &trigger.name,
                ) {
                    Ok(output) => self.apply_script_output(entity, output),
                    Err(e) => eprintln!("[Script ERROR] {e}"),
                }
            }
        }

        // Fire on_animation_end for finished one-shot animations
        let anim_ended = self.last_step.animation_ended.clone();
        for (entity, state_name) in anim_ended {
            let Some(behavior) = self.engine.active_scene.behavior(entity).cloned() else {
                continue;
            };
            if !behavior.enabled {
                continue;
            }
            let ctx = script_context(&self.engine, entity, &self.input);
            match self.script_runtime.call_on_animation_end(
                &behavior.script_path,
                &ctx,
                &state_name,
            ) {
                Ok(output) => self.apply_script_output(entity, output),
                Err(e) => eprintln!("[Script ERROR] {e}"),
            }
        }

        // Fire on_collision for solid collision events
        let collisions = self.last_step.solid_collisions.clone();
        for collision in collisions {
            let entity = collision.entity;
            let Some(behavior) = self.engine.active_scene.behavior(entity).cloned() else {
                continue;
            };
            if !behavior.enabled {
                continue;
            }
            let (other_id, other_tag, normal_x, normal_y) = match collision.obstacle {
                crab2d_core::SolidObstacle::Entity(other) => {
                    let other_tag = self
                        .engine
                        .active_scene
                        .tag(other)
                        .map(|t| t.tag.clone())
                        .unwrap_or_default();
                    let normal_x = if collision.axis == crab2d_core::CollisionAxis::X {
                        1.0_f64
                    } else {
                        0.0_f64
                    };
                    let normal_y = if collision.axis == crab2d_core::CollisionAxis::Y {
                        1.0_f64
                    } else {
                        0.0_f64
                    };
                    (other.raw() as i64, other_tag, normal_x, normal_y)
                }
                crab2d_core::SolidObstacle::Tile { .. } => {
                    let normal_x = if collision.axis == crab2d_core::CollisionAxis::X {
                        1.0_f64
                    } else {
                        0.0_f64
                    };
                    let normal_y = if collision.axis == crab2d_core::CollisionAxis::Y {
                        1.0_f64
                    } else {
                        0.0_f64
                    };
                    (-1_i64, "tile".to_string(), normal_x, normal_y)
                }
            };
            let ctx = script_context(&self.engine, entity, &self.input);
            match self.script_runtime.call_on_collision(
                &behavior.script_path,
                &ctx,
                other_id,
                &other_tag,
                normal_x,
                normal_y,
            ) {
                Ok(output) => self.apply_script_output(entity, output),
                Err(e) => eprintln!("[Script ERROR] {e}"),
            }
        }
    }

    fn play_auto_audio(&mut self) {
        let clips: Vec<(String, f32, bool)> = self
            .engine
            .active_scene
            .audios()
            .filter(|(_, a)| a.auto_play)
            .map(|(_, a)| (a.clip_path.clone(), a.volume, a.looping))
            .collect();
        for (path, volume, looping) in clips {
            self.audio_system.play_clip_once(&path, volume, looping);
        }
    }

    fn apply_script_output(&mut self, entity: EntityId, output: ScriptOutput) {
        if let Some(vel) = self.engine.active_scene.velocity_mut(entity) {
            if let Some(x) = output.set_vel_x {
                vel.linear.x = x;
            }
            if let Some(y) = output.set_vel_y {
                vel.linear.y = y;
            }
        }
        if let Some(node) = self.engine.active_scene.node_mut(entity) {
            if let Some(x) = output.set_pos_x {
                node.transform.position.x = x;
            }
            if let Some(y) = output.set_pos_y {
                node.transform.position.y = y;
            }
        }
        if output.destroy_self {
            let _ = self.engine.active_scene.despawn_node(entity);
        }
        if let Some(path) = output.load_scene {
            self.scene_manager.load_scene(path);
        }
        if let Some(state) = output.set_anim_state {
            if let Some(anim) = self.engine.active_scene.animation_mut(entity) {
                anim.set_state(&state);
                anim.playing = true;
            }
        }
        if let Some(clip) = output.play_audio {
            let path = clip.clone();
            self.audio_system.play_clip_once(&path, 1.0, false);
        }
        if let Some(text) = output.set_text {
            if let Some(wt) = self.engine.active_scene.world_text_mut(entity) {
                wt.text = text.clone();
            }
            if let Some(label) = self.engine.active_scene.ui_label_mut(entity) {
                label.text = text;
            }
        }
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
        if self.input.was_key_pressed(KeyCode::F1) {
            self.debug_overlay = !self.debug_overlay;
        }

        // Track FPS
        if delta_seconds > 0.0 {
            if self.fps_history.len() >= 60 {
                self.fps_history.pop_front();
            }
            self.fps_history.push_back(1.0 / delta_seconds);
        }

        match self.engine.tick_with_input(delta_seconds, &self.input) {
            Ok(step) => self.last_step = step,
            Err(error) => eprintln!("Runtime tick failed: {error}"),
        }

        let anim_ended =
            animation_system::tick_animations(&mut self.engine.active_scene, delta_seconds);
        self.last_step.animation_ended = anim_ended;
        self.particle_system
            .tick(&self.engine.active_scene, delta_seconds);
        self.run_script_frame(delta_seconds);
        self.play_auto_audio();

        if let Some((new_scene, _path)) = self.scene_manager.apply_transition() {
            self.engine.active_scene = new_scene;
            self.scripts_started.clear();
            self.script_runtime.unload_all();
        }

        let entity_count = self.engine.active_scene.len();
        let avg_fps = if self.fps_history.is_empty() {
            0.0
        } else {
            self.fps_history.iter().sum::<f32>() / self.fps_history.len() as f32
        };
        self.renderer.draw(
            ui,
            &self.engine.active_scene,
            &self.last_step,
            &self.particle_system,
            self.debug_overlay,
            avg_fps,
            entity_count,
        );
        ctx.request_repaint();
    }
}

fn runtime_keys() -> [(egui::Key, KeyCode); 14] {
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
        (egui::Key::Space, KeyCode::Space),
        (egui::Key::Enter, KeyCode::Enter),
        (egui::Key::F1, KeyCode::F1),
        (egui::Key::F2, KeyCode::F2),
        (egui::Key::F3, KeyCode::F3),
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

    #[allow(clippy::too_many_arguments)]
    fn draw(
        &mut self,
        ui: &mut egui::Ui,
        scene: &Scene,
        frame_step: &FrameStep,
        particle_system: &ParticleSystem,
        debug_overlay: bool,
        avg_fps: f32,
        entity_count: usize,
    ) {
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

        // Draw particles
        for (entity_id, emitter) in scene.particle_emitters().collect::<Vec<_>>() {
            if let Some(state) = particle_system.get_state(entity_id) {
                for particle in &state.particles {
                    let progress = particle.progress();
                    let cr = lerp_u8(emitter.color_start[0], emitter.color_end[0], progress);
                    let cg = lerp_u8(emitter.color_start[1], emitter.color_end[1], progress);
                    let cb = lerp_u8(emitter.color_start[2], emitter.color_end[2], progress);
                    let ca = lerp_u8(emitter.color_start[3], emitter.color_end[3], progress);
                    let size =
                        emitter.size_start + (emitter.size_end - emitter.size_start) * progress;
                    let screen_pos = world_to_screen(rect, &render_list, particle.position);
                    painter.circle_filled(
                        screen_pos,
                        size / 2.0,
                        egui::Color32::from_rgba_unmultiplied(cr, cg, cb, ca),
                    );
                }
            }
        }

        // Draw in-game UI labels
        for (_entity_id, label) in scene.ui_labels().collect::<Vec<_>>() {
            if !label.visible {
                continue;
            }
            let [r, g, b, a] = label.color_rgba;
            let anchor_pos = resolve_anchor(label.anchor, rect);
            let pos = egui::pos2(anchor_pos.x + label.offset_x, anchor_pos.y + label.offset_y);
            painter.text(
                pos,
                egui::Align2::LEFT_TOP,
                &label.text,
                egui::FontId::proportional(label.font_size),
                egui::Color32::from_rgba_unmultiplied(r, g, b, a),
            );
        }

        // Draw in-game UI panels
        for (_entity_id, panel) in scene.ui_panels().collect::<Vec<_>>() {
            if !panel.visible {
                continue;
            }
            let [r, g, b, a] = panel.color_rgba;
            let anchor_pos = resolve_anchor(panel.anchor, rect);
            let panel_rect = egui::Rect::from_min_size(
                egui::pos2(anchor_pos.x + panel.offset_x, anchor_pos.y + panel.offset_y),
                egui::vec2(panel.width, panel.height),
            );
            painter.rect_filled(
                panel_rect,
                4.0,
                egui::Color32::from_rgba_unmultiplied(r, g, b, a),
            );
        }

        // Draw world-space text labels
        for (entity_id, wt) in scene.world_texts().collect::<Vec<_>>() {
            if !wt.visible {
                continue;
            }
            let Some(node) = scene.node(entity_id) else {
                continue;
            };
            let world_pos = Vec2::new(
                node.transform.position.x + wt.offset_x,
                node.transform.position.y + wt.offset_y,
            );
            let screen_pos = world_to_screen(rect, &render_list, world_pos);
            let [r, g, b, a] = wt.color_rgba;
            painter.text(
                screen_pos,
                egui::Align2::CENTER_BOTTOM,
                &wt.text,
                egui::FontId::proportional(wt.font_size),
                egui::Color32::from_rgba_unmultiplied(r, g, b, a),
            );
        }

        // Draw collider wireframes in debug mode
        if debug_overlay {
            for (entity, collider) in scene.colliders().collect::<Vec<_>>() {
                if let Some(node) = scene.node(entity) {
                    let aabb = collider.world_aabb(node.transform);
                    let min = world_to_screen(rect, &render_list, aabb.min);
                    let max = world_to_screen(rect, &render_list, aabb.max);
                    let wire_rect = egui::Rect::from_min_max(min, max);
                    let color = if collider.is_sensor {
                        egui::Color32::from_rgba_unmultiplied(80, 220, 120, 180)
                    } else {
                        egui::Color32::from_rgba_unmultiplied(220, 80, 80, 180)
                    };
                    painter.rect_stroke(
                        wire_rect,
                        0.0,
                        egui::Stroke::new(1.5, color),
                        egui::StrokeKind::Outside,
                    );
                }
            }
        }

        self.draw_overlay(
            &painter,
            rect,
            frame_step,
            debug_overlay,
            avg_fps,
            entity_count,
        );
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

    fn draw_overlay(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        frame_step: &FrameStep,
        debug_overlay: bool,
        avg_fps: f32,
        entity_count: usize,
    ) {
        if debug_overlay {
            let lines = [
                format!("FPS: {avg_fps:.0}"),
                format!("Entities: {entity_count}"),
                format!("Collisions: {}", frame_step.solid_collisions.len()),
                format!("Triggers: {}", frame_step.triggers.len()),
                "Press F1 to hide debug".to_string(),
            ];
            let line_height = 18.0;
            let bg_h = lines.len() as f32 * line_height + 12.0;
            let bg = egui::Rect::from_min_size(
                rect.left_top() + egui::vec2(12.0, 12.0),
                egui::vec2(200.0, bg_h),
            );
            painter.rect_filled(bg, 5.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200));
            for (i, line) in lines.iter().enumerate() {
                painter.text(
                    bg.left_top() + egui::vec2(10.0, 8.0 + i as f32 * line_height),
                    egui::Align2::LEFT_TOP,
                    line,
                    egui::FontId::monospace(12.0),
                    egui::Color32::from_rgb(230, 238, 240),
                );
            }
        } else {
            let text = format!("Crab2D  |  {avg_fps:.0} FPS  |  [F1] debug",);
            let bg = egui::Rect::from_min_size(
                rect.left_top() + egui::vec2(12.0, 12.0),
                egui::vec2(220.0, 26.0),
            );
            painter.rect_filled(bg, 5.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 140));
            painter.text(
                bg.left_center() + egui::vec2(10.0, 0.0),
                egui::Align2::LEFT_CENTER,
                text,
                egui::FontId::monospace(12.0),
                egui::Color32::from_rgb(180, 190, 200),
            );
        }
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

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t.clamp(0.0, 1.0)) as u8
}

fn world_to_screen(viewport: egui::Rect, render_list: &RenderList, world_pos: Vec2) -> egui::Pos2 {
    let camera_position = render_list
        .camera
        .map(|camera| camera.transform.position)
        .unwrap_or(Vec2::ZERO);
    let zoom = render_list.camera.map(|camera| camera.zoom).unwrap_or(1.0);
    let relative = world_pos - camera_position;
    viewport.center() + egui::vec2(relative.x * zoom, -relative.y * zoom)
}

fn script_context(engine: &Engine, entity: EntityId, input: &InputState) -> ScriptContext {
    let node = engine.active_scene.node(entity);
    let velocity = engine.active_scene.velocity(entity);
    let anim_state = engine
        .active_scene
        .animation(entity)
        .map(|a| a.current_state.clone())
        .unwrap_or_default();
    let (mouse_world_x, mouse_world_y) = {
        let cursor = input.cursor_position().unwrap_or((0.0, 0.0));
        // Simple world-space approximation using camera position and zoom
        // (real conversion would need viewport size; this is a best-effort)
        (cursor.0, cursor.1)
    };
    let (scroll_x, scroll_y) = input.scroll_delta();
    ScriptContext {
        entity_id: entity.raw(),
        pos_x: node.map(|n| n.transform.position.x).unwrap_or(0.0),
        pos_y: node.map(|n| n.transform.position.y).unwrap_or(0.0),
        vel_x: velocity.map(|v| v.linear.x).unwrap_or(0.0),
        vel_y: velocity.map(|v| v.linear.y).unwrap_or(0.0),
        tag: engine
            .active_scene
            .tag(entity)
            .map(|t| t.tag.clone())
            .unwrap_or_default(),
        keys_pressed: input
            .pressed_keys()
            .map(key_name)
            .filter(|s| !s.is_empty())
            .collect(),
        anim_state,
        mouse_world_x,
        mouse_world_y,
        mouse_left: input.is_mouse_down(MouseButton::Left),
        mouse_right: input.is_mouse_down(MouseButton::Right),
        mouse_middle: input.is_mouse_down(MouseButton::Middle),
        scroll_x,
        scroll_y,
    }
}

fn key_name(key: KeyCode) -> String {
    match key {
        KeyCode::Character(c) => c.to_string(),
        KeyCode::ArrowUp => "arrow_up".to_owned(),
        KeyCode::ArrowDown => "arrow_down".to_owned(),
        KeyCode::ArrowLeft => "arrow_left".to_owned(),
        KeyCode::ArrowRight => "arrow_right".to_owned(),
        KeyCode::Escape => "escape".to_owned(),
        KeyCode::Space => "space".to_owned(),
        KeyCode::Enter => "enter".to_owned(),
        KeyCode::F1 => "f1".to_owned(),
        KeyCode::F2 => "f2".to_owned(),
        KeyCode::F3 => "f3".to_owned(),
        KeyCode::F4 => "f4".to_owned(),
        KeyCode::F5 => "f5".to_owned(),
    }
}

fn resolve_anchor(anchor: UiAnchor, rect: egui::Rect) -> egui::Pos2 {
    match anchor {
        UiAnchor::TopLeft => rect.left_top(),
        UiAnchor::TopCenter => egui::pos2(rect.center().x, rect.top()),
        UiAnchor::TopRight => rect.right_top(),
        UiAnchor::MiddleLeft => egui::pos2(rect.left(), rect.center().y),
        UiAnchor::Center => rect.center(),
        UiAnchor::MiddleRight => egui::pos2(rect.right(), rect.center().y),
        UiAnchor::BottomLeft => rect.left_bottom(),
        UiAnchor::BottomCenter => egui::pos2(rect.center().x, rect.bottom()),
        UiAnchor::BottomRight => rect.right_bottom(),
    }
}
