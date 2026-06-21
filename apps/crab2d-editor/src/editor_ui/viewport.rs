use super::*;

impl Crab2DEditorUi {
    pub(super) fn show_viewport(&mut self, root: &mut egui::Ui) {
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

        let world_texts: Vec<(EntityId, Vec2, String)> = self
            .app
            .scene_nodes()
            .iter()
            .filter_map(|node| {
                self.app.node_world_text(node.id).map(|wt| {
                    (
                        node.id,
                        Vec2::new(
                            node.transform.position.x + wt.offset_x,
                            node.transform.position.y + wt.offset_y,
                        ),
                        wt.text.clone(),
                    )
                })
            })
            .collect();

        let mut clicked_id = None;
        let mut clicked_name = None;
        let mut paint_request = None;
        let mut asset_warning = None;
        let theme = theme();

        // Capture pan/zoom before closure
        let pan = self.viewport_pan;
        let zoom = self.viewport_zoom.max(0.05);

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme.colors.viewport_bg))
            .show_inside(root, |ui| {
                let available = ui.available_size();
                let (rect, response) =
                    ui.allocate_exact_size(available, egui::Sense::click_and_drag());
                let painter = ui.painter_at(rect);

                let origin = rect.center() + pan;
                let world_to_screen =
                    |position: Vec2| origin + egui::vec2(position.x * zoom, -position.y * zoom);
                let screen_to_world = |position: egui::Pos2| {
                    Vec2::new(
                        (position.x - origin.x) / zoom,
                        -(position.y - origin.y) / zoom,
                    )
                };

                let (grid_world_step, grid_screen_step) = viewport_grid_step(zoom);

                painter.rect_filled(rect, 0.0, theme.colors.viewport_bg);
                if self.show_grid {
                    draw_world_grid(
                        &painter,
                        rect,
                        origin,
                        grid_screen_step,
                        theme.colors.grid_minor,
                    );
                    draw_world_grid(
                        &painter,
                        rect,
                        origin,
                        grid_screen_step * 4.0,
                        theme.colors.grid_major,
                    );
                    draw_world_axes(&painter, rect, origin);
                }

                let mut hit_rects = Vec::new();

                for item in &items {
                    draw_camera_frame(&painter, rect, &world_to_screen, zoom, item);
                }

                for item in &items {
                    if let Some(tilemap) = &item.tilemap {
                        let (tilemap_rect, warning) = self.draw_tilemap(
                            &painter,
                            rect,
                            &world_to_screen,
                            zoom,
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
                        self.draw_node(ui.ctx(), &painter, &world_to_screen, zoom, item);
                    if asset_warning.is_none() {
                        asset_warning = warning;
                    }
                    hit_rects.push((item.id, hit_rect));
                }

                for item in &items {
                    draw_collider_overlay(&painter, &world_to_screen, item);
                }

                // Draw WorldText labels in viewport
                for (_entity, world_pos, text) in &world_texts {
                    let screen_pos = world_to_screen(*world_pos);
                    painter.text(
                        screen_pos,
                        egui::Align2::CENTER_BOTTOM,
                        text.as_str(),
                        egui::FontId::proportional((12.0 * zoom).clamp(9.0, 34.0)),
                        egui::Color32::from_rgba_unmultiplied(255, 240, 100, 220),
                    );
                }

                let hover_pos = ui.input(|i| i.pointer.hover_pos());
                if let Some(pos) = hover_pos.filter(|pos| rect.contains(*pos)) {
                    if self.asset_drag.is_some() {
                        self.draw_asset_drag_preview(&painter, pos);
                    }
                }

                if self.asset_drag.is_some() && ui.input(|i| i.pointer.primary_released()) {
                    if let Some(pos) = hover_pos.filter(|pos| rect.contains(*pos)) {
                        self.finish_asset_drag_at(screen_to_world(pos));
                    } else {
                        self.cancel_asset_drag();
                    }
                }

                // Middle-mouse drag, or primary drag while the Pan tool is active.
                if response.dragged_by(egui::PointerButton::Middle)
                    || (self.active_tool == EditorTool::Pan
                        && response.dragged_by(egui::PointerButton::Primary))
                {
                    let delta = ui.input(|input| input.pointer.delta());
                    self.viewport_pan += delta;
                }

                if self.asset_drag.is_some() || self.viewport_drag.is_some() {
                    if let Some(pos) = hover_pos.filter(|pos| rect.contains(*pos)) {
                        self.viewport_pan += edge_pan_delta(rect, pos);
                    }
                }

                // Scroll → zoom
                let scroll = ui.input(|i| i.smooth_scroll_delta.y);
                if scroll != 0.0 && hover_pos.is_some_and(|pos| rect.contains(pos)) {
                    let hover_pos = hover_pos.unwrap_or(rect.center());
                    let before_world = screen_to_world(hover_pos);
                    let factor = if scroll > 0.0 { 1.1_f32 } else { 1.0 / 1.1 };
                    let new_zoom = (self.viewport_zoom * factor).clamp(0.08, 24.0);
                    self.viewport_zoom = new_zoom;
                    self.viewport_pan = hover_pos
                        - rect.center()
                        - egui::vec2(before_world.x * new_zoom, -before_world.y * new_zoom);
                }

                // Reset pan/zoom with Home key
                if ui.input(|i| i.key_pressed(egui::Key::Home)) {
                    self.viewport_pan = egui::Vec2::ZERO;
                    self.viewport_zoom = 1.0;
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
                            EditorTool::Pan => {}
                        }
                    }
                }

                if matches!(
                    self.active_tool,
                    EditorTool::TileBrush | EditorTool::EraseTile
                ) && response.dragged_by(egui::PointerButton::Primary)
                {
                    if let Some(pos) = response.interact_pointer_pos() {
                        if rect.contains(pos) {
                            paint_request = Some(screen_to_world(pos));
                        }
                    }
                }

                if response.secondary_clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        if rect.contains(pos) {
                            self.viewport_context_world = Some(screen_to_world(pos));
                            if let Some((id, _)) = hit_rects
                                .iter()
                                .rev()
                                .find(|(_, hit)| hit.expand(5.0).contains(pos))
                            {
                                self.select_node(*id);
                            }
                        }
                    }
                }
                response.context_menu(|ui| {
                    self.show_viewport_context_menu(ui);
                });

                if self.active_tool == EditorTool::Select {
                    if response.drag_started_by(egui::PointerButton::Primary) {
                        if let Some(pos) = response.interact_pointer_pos() {
                            let selected_resize = selected_hit_rect(&hit_rects, self.selected)
                                .and_then(|rect| resize_handle_at(rect.expand(5.0), pos));

                            if let (Some(entity), Some(handle)) = (self.selected, selected_resize) {
                                if let Some(before) = self.app.node_transform(entity) {
                                    let start_size = selected_hit_rect(&hit_rects, Some(entity))
                                        .map(|rect| rect.size())
                                        .unwrap_or(egui::vec2(32.0, 32.0));
                                    self.viewport_drag =
                                        Some(ViewportDrag::Scale(ViewportScaleDrag {
                                            entity,
                                            before,
                                            start_pointer: pos,
                                            start_size,
                                            handle,
                                        }));
                                }
                            } else if let Some((id, _)) = hit_rects
                                .iter()
                                .rev()
                                .find(|(_, hit)| hit.expand(5.0).contains(pos))
                            {
                                self.select_node(*id);
                                if let Some(before) = self.app.node_transform(*id) {
                                    self.viewport_drag = Some(ViewportDrag::Move {
                                        entity: *id,
                                        before,
                                    });
                                }
                            }
                        }
                    }

                    if response.dragged_by(egui::PointerButton::Primary) {
                        self.apply_viewport_drag(response.drag_delta(), zoom, grid_world_step);
                    }

                    if response.drag_stopped_by(egui::PointerButton::Primary) {
                        if let Some(drag) = self.viewport_drag.take() {
                            let entity = drag.entity();
                            let before = drag.before();
                            if let Some(after) = self.app.node_transform(entity) {
                                self.app.record_move_node(entity, before, after);
                                self.sync_selected_buffers();
                                self.set_status(match drag {
                                    ViewportDrag::Move { .. } => "Node moved",
                                    ViewportDrag::Scale(_) => "Node resized",
                                });
                            }
                        }
                    }
                }

                self.show_viewport_overlays(
                    ui,
                    rect,
                    asset_warning.as_deref(),
                    self.viewport_zoom,
                    grid_world_step,
                    hover_pos.map(screen_to_world),
                );
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
}

fn selected_hit_rect(
    hit_rects: &[(EntityId, egui::Rect)],
    selected: Option<EntityId>,
) -> Option<egui::Rect> {
    let selected = selected?;
    hit_rects
        .iter()
        .rev()
        .find(|(id, _)| *id == selected)
        .map(|(_, rect)| *rect)
}

fn resize_handle_at(rect: egui::Rect, pos: egui::Pos2) -> Option<ResizeHandle> {
    resize_handle_rects(rect)
        .into_iter()
        .find(|(_, handle_rect)| handle_rect.contains(pos))
        .map(|(handle, _)| handle)
}

fn resize_handle_rects(rect: egui::Rect) -> [(ResizeHandle, egui::Rect); 4] {
    [
        (
            ResizeHandle::TOP_LEFT,
            egui::Rect::from_center_size(rect.left_top(), egui::vec2(12.0, 12.0)),
        ),
        (
            ResizeHandle::TOP_RIGHT,
            egui::Rect::from_center_size(rect.right_top(), egui::vec2(12.0, 12.0)),
        ),
        (
            ResizeHandle::BOTTOM_LEFT,
            egui::Rect::from_center_size(rect.left_bottom(), egui::vec2(12.0, 12.0)),
        ),
        (
            ResizeHandle::BOTTOM_RIGHT,
            egui::Rect::from_center_size(rect.right_bottom(), egui::vec2(12.0, 12.0)),
        ),
    ]
}

fn edge_pan_delta(rect: egui::Rect, pos: egui::Pos2) -> egui::Vec2 {
    let edge = 42.0;
    let speed = 14.0;
    let mut delta = egui::Vec2::ZERO;

    if pos.x < rect.left() + edge {
        delta.x += speed;
    } else if pos.x > rect.right() - edge {
        delta.x -= speed;
    }

    if pos.y < rect.top() + edge {
        delta.y += speed;
    } else if pos.y > rect.bottom() - edge {
        delta.y -= speed;
    }

    delta
}
