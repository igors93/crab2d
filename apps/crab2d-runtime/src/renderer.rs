use std::collections::BTreeMap;
use std::path::PathBuf;

use crab2d_core::{particle_system::ParticleSystem, FrameStep};
use crab2d_render::{RenderItem, RenderList, SpriteRenderCommand, TilemapRenderCommand};
use crab2d_scene::{Scene, UiAnchor, Vec2};
use eframe::egui;

use crate::assets::{normalize_asset_path, resolve_path};

pub(crate) struct EguiRuntimeRenderer {
    asset_roots: Vec<PathBuf>,
    textures: BTreeMap<String, egui::TextureHandle>,
}

impl EguiRuntimeRenderer {
    pub(crate) fn new(asset_roots: Vec<PathBuf>) -> Self {
        Self {
            asset_roots,
            textures: BTreeMap::new(),
        }
    }

    pub(crate) fn configure(&mut self, ctx: &egui::Context) {
        let mut style = (*ctx.global_style()).clone();
        style.visuals = egui::Visuals::dark();
        style.visuals.panel_fill = egui::Color32::from_rgb(10, 12, 14);
        ctx.set_global_style(style);
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw(
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

        self.draw_particles(scene, particle_system, &painter, rect, &render_list);
        self.draw_ui(scene, &painter, rect);
        self.draw_world_text(scene, &painter, rect, &render_list);

        if debug_overlay {
            self.draw_colliders(scene, &painter, rect, &render_list);
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

    fn draw_particles(
        &self,
        scene: &Scene,
        particle_system: &ParticleSystem,
        painter: &egui::Painter,
        viewport: egui::Rect,
        render_list: &RenderList,
    ) {
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
                    let screen_pos = world_to_screen(viewport, render_list, particle.position);
                    painter.circle_filled(
                        screen_pos,
                        size / 2.0,
                        egui::Color32::from_rgba_unmultiplied(cr, cg, cb, ca),
                    );
                }
            }
        }
    }

    fn draw_ui(&self, scene: &Scene, painter: &egui::Painter, viewport: egui::Rect) {
        for (_entity_id, label) in scene.ui_labels().collect::<Vec<_>>() {
            if !label.visible {
                continue;
            }
            let [r, g, b, a] = label.color_rgba;
            let anchor_pos = resolve_anchor(label.anchor, viewport);
            let pos = egui::pos2(anchor_pos.x + label.offset_x, anchor_pos.y + label.offset_y);
            painter.text(
                pos,
                egui::Align2::LEFT_TOP,
                &label.text,
                egui::FontId::proportional(label.font_size),
                egui::Color32::from_rgba_unmultiplied(r, g, b, a),
            );
        }

        for (_entity_id, panel) in scene.ui_panels().collect::<Vec<_>>() {
            if !panel.visible {
                continue;
            }
            let [r, g, b, a] = panel.color_rgba;
            let anchor_pos = resolve_anchor(panel.anchor, viewport);
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
    }

    fn draw_world_text(
        &self,
        scene: &Scene,
        painter: &egui::Painter,
        viewport: egui::Rect,
        render_list: &RenderList,
    ) {
        for (entity_id, world_text) in scene.world_texts().collect::<Vec<_>>() {
            if !world_text.visible {
                continue;
            }
            let Some(node) = scene.node(entity_id) else {
                continue;
            };
            let world_pos = Vec2::new(
                node.transform.position.x + world_text.offset_x,
                node.transform.position.y + world_text.offset_y,
            );
            let screen_pos = world_to_screen(viewport, render_list, world_pos);
            let [r, g, b, a] = world_text.color_rgba;
            painter.text(
                screen_pos,
                egui::Align2::CENTER_BOTTOM,
                &world_text.text,
                egui::FontId::proportional(world_text.font_size),
                egui::Color32::from_rgba_unmultiplied(r, g, b, a),
            );
        }
    }

    fn draw_colliders(
        &self,
        scene: &Scene,
        painter: &egui::Painter,
        viewport: egui::Rect,
        render_list: &RenderList,
    ) {
        for (entity, collider) in scene.colliders().collect::<Vec<_>>() {
            if let Some(node) = scene.node(entity) {
                let aabb = collider.world_aabb(node.transform);
                let min = world_to_screen(viewport, render_list, aabb.min);
                let max = world_to_screen(viewport, render_list, aabb.max);
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
            let text = format!("Crab2D  |  {avg_fps:.0} FPS  |  [F1] debug");
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
