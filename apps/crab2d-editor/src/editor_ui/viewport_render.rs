use super::*;

impl Crab2DEditorUi {
    pub(super) fn draw_asset_drag_preview(
        &mut self,
        painter: &egui::Painter,
        screen_pos: egui::Pos2,
    ) {
        let Some(drag) = self.asset_drag.clone() else {
            return;
        };
        let theme = theme();
        let rect = egui::Rect::from_center_size(screen_pos, egui::vec2(54.0, 54.0));
        painter.rect_filled(rect, theme.radius.md, theme.colors.viewport_overlay);
        painter.rect_stroke(
            rect,
            theme.radius.md,
            egui::Stroke::new(1.5, theme.colors.accent),
            egui::StrokeKind::Inside,
        );

        match self.textures.load(painter.ctx(), drag.asset_path.as_str()) {
            TextureLookup::Loaded(texture) => {
                let image_rect = fit_rect(texture.size_vec2(), rect.shrink(8.0));
                painter.image(
                    texture.id(),
                    image_rect,
                    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, 210),
                );
            }
            TextureLookup::Failed(_) | TextureLookup::Missing => {
                draw_missing_texture_marker(painter, rect.shrink(10.0), "?");
            }
        }

        draw_node_label(
            painter,
            rect.center_bottom() + egui::vec2(0.0, 8.0),
            drag.display_name.as_str(),
        );
    }

    pub(super) fn draw_tilemap(
        &mut self,
        painter: &egui::Painter,
        viewport_rect: egui::Rect,
        world_to_screen: &dyn Fn(Vec2) -> egui::Pos2,
        viewport_zoom: f32,
        item: &NodeView,
        tilemap: &TilemapComponent,
    ) -> (egui::Rect, Option<String>) {
        let theme = theme();
        let tile_width = tilemap.tile_size.width as f32;
        let tile_height = tilemap.tile_size.height as f32;
        let scale_x = item.transform.scale.x.abs().max(0.05);
        let scale_y = item.transform.scale.y.abs().max(0.05);
        let origin = item.transform.position;
        let map_size = egui::vec2(
            tilemap.map_size.width as f32 * tile_width * scale_x,
            tilemap.map_size.height as f32 * tile_height * scale_y,
        );
        let map_center = world_to_screen(Vec2::new(
            origin.x + map_size.x / 2.0,
            origin.y + map_size.y / 2.0,
        ));
        let map_rect = egui::Rect::from_center_size(map_center, map_size * viewport_zoom);
        let mut texture_error = None;
        let tileset_texture = tilemap.tileset.as_ref().and_then(|tileset| {
            match self.textures.load(painter.ctx(), &tileset.image_path) {
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
                origin.x + (visible.x as f32 * tile_width + tile_width / 2.0) * scale_x,
                origin.y + (visible.y as f32 * tile_height + tile_height / 2.0) * scale_y,
            );
            let tile_rect = egui::Rect::from_center_size(
                world_to_screen(world_center),
                egui::vec2(
                    tile_width * scale_x * viewport_zoom,
                    tile_height * scale_y * viewport_zoom,
                ),
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

    pub(super) fn draw_node(
        &mut self,
        ctx: &egui::Context,
        painter: &egui::Painter,
        world_to_screen: &dyn Fn(Vec2) -> egui::Pos2,
        viewport_zoom: f32,
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
                        (size.x * item.transform.scale.x.abs().max(0.1) * viewport_zoom)
                            .clamp(8.0, 1024.0),
                        (size.y * item.transform.scale.y.abs().max(0.1) * viewport_zoom)
                            .clamp(8.0, 1024.0),
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
                    let marker_size = (42.0 * viewport_zoom).clamp(18.0, 84.0);
                    let rect =
                        egui::Rect::from_center_size(center, egui::vec2(marker_size, marker_size));
                    draw_missing_texture_marker(painter, rect, "!");
                    warning = Some(format!("Sprite asset failed for '{}': {error}", item.name));
                    rect
                }
                TextureLookup::Missing => {
                    let marker_size = (42.0 * viewport_zoom).clamp(18.0, 84.0);
                    let rect =
                        egui::Rect::from_center_size(center, egui::vec2(marker_size, marker_size));
                    draw_node_marker(painter, center, color, marker_size);
                    rect
                }
            }
        } else {
            let marker_size = (34.0 * viewport_zoom).clamp(18.0, 72.0);
            let rect = egui::Rect::from_center_size(center, egui::vec2(marker_size, marker_size));
            draw_node_marker(painter, center, color, marker_size);
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

    pub(super) fn paint_tile_at(&mut self, world_position: Vec2) {
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

        let scale_x = node.transform.scale.x.abs().max(0.05);
        let scale_y = node.transform.scale.y.abs().max(0.05);
        let x = (local_x / (tilemap.tile_size.width as f32 * scale_x)).floor() as u32;
        let y = (local_y / (tilemap.tile_size.height as f32 * scale_y)).floor() as u32;
        if x >= tilemap.map_size.width || y >= tilemap.map_size.height {
            return;
        }

        let tile = match self.active_tool {
            EditorTool::TileBrush => Some(TileCell::new(self.selected_tile_index)),
            EditorTool::EraseTile => None,
            EditorTool::Select | EditorTool::Pan => return,
        };

        match self
            .app
            .execute_command_with_history(EditorCommand::set_tile(
                entity,
                self.active_layer.clone(),
                x,
                y,
                tile,
            )) {
            Ok(_) => self.set_status(format!("Tile ({x}, {y}) updated")),
            Err(error) => self.set_error(format!("{error}")),
        }
    }
}
