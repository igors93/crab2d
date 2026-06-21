use super::*;

impl Crab2DEditorUi {
    pub(super) fn show_viewport_overlays(
        &mut self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        asset_warning: Option<&str>,
        zoom: f32,
        grid_world_step: f32,
        hover_world: Option<Vec2>,
    ) {
        let theme = theme();

        egui::Area::new(egui::Id::new("viewport_status_overlay"))
            .order(egui::Order::Foreground)
            .fixed_pos(rect.left_top() + egui::vec2(14.0, 18.0))
            .show(ui.ctx(), |ui| {
                egui::Frame::new()
                    .fill(theme.colors.viewport_overlay)
                    .stroke(egui::Stroke::new(1.0, theme.colors.border))
                    .corner_radius(theme.radius.sm)
                    .inner_margin(egui::Margin::symmetric(8, 4))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = theme.spacing.sm;
                            self.show_tool_button(ui, EditorTool::Select, "Select", "Select nodes");
                            self.show_tool_button(ui, EditorTool::Pan, "Pan", "Pan viewport");
                            self.show_tool_button(
                                ui,
                                EditorTool::TileBrush,
                                "Brush",
                                "Paint selected tile",
                            );
                            self.show_tool_button(
                                ui,
                                EditorTool::EraseTile,
                                "Erase",
                                "Erase tiles",
                            );
                            ui.separator();
                            widgets::chip(
                                ui,
                                format!("Tile {}", self.selected_tile_index).as_str(),
                                StatusTone::Info,
                            )
                            .on_hover_text("Selected tile index for painting");
                            ui.separator();
                            if widgets::icon_button(ui, "-", "Zoom out", true).clicked() {
                                self.viewport_zoom = (self.viewport_zoom / 1.15).clamp(0.08, 24.0);
                            }
                            if widgets::toolbar_button(
                                ui,
                                format!("{:.0}%", zoom * 100.0).as_str(),
                                "Reset zoom",
                                true,
                                false,
                            )
                            .clicked()
                            {
                                self.viewport_zoom = 1.0;
                            }
                            if widgets::icon_button(ui, "+", "Zoom in", true).clicked() {
                                self.viewport_zoom = (self.viewport_zoom * 1.15).clamp(0.08, 24.0);
                            }
                            ui.separator();
                            ui.toggle_value(&mut self.show_grid, "Grid")
                                .on_hover_text(format!("Grid step: {:.0}", grid_world_step));
                            ui.toggle_value(&mut self.snap_enabled, "Snap")
                                .on_hover_text("Snap movement and scale to grid");
                            egui::ComboBox::from_id_salt("viewport_layer_combo")
                                .selected_text(self.active_layer.as_str())
                                .width(120.0)
                                .show_ui(ui, |ui| {
                                    for layer in self.tilemap_layer_names() {
                                        ui.selectable_value(
                                            &mut self.active_layer,
                                            layer.clone(),
                                            layer,
                                        );
                                    }
                                });
                        });
                    });
            });

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

        egui::Area::new(egui::Id::new("viewport_coords_overlay"))
            .order(egui::Order::Foreground)
            .fixed_pos(rect.left_bottom() + egui::vec2(14.0, -34.0))
            .show(ui.ctx(), |ui| {
                egui::Frame::new()
                    .fill(theme.colors.viewport_overlay)
                    .stroke(egui::Stroke::new(1.0, theme.colors.border))
                    .corner_radius(theme.radius.sm)
                    .inner_margin(egui::Margin::symmetric(8, 3))
                    .show(ui, |ui| {
                        let position_text = hover_world
                            .map(|pos| format!("Pos ({:.0}, {:.0})", pos.x, pos.y))
                            .unwrap_or_else(|| "Pos (-, -)".to_owned());
                        ui.label(
                            egui::RichText::new(format!(
                                "{}  |  Layer {}",
                                position_text, self.active_layer
                            ))
                            .size(10.5)
                            .color(theme.colors.text_muted),
                        );
                    });
            });
    }

    pub(super) fn apply_viewport_drag(
        &mut self,
        pointer_pos: egui::Pos2,
        zoom: f32,
        grid_world_step: f32,
    ) {
        let Some(drag) = self.viewport_drag else {
            return;
        };

        let result = match drag {
            ViewportDrag::Move {
                before,
                start_pointer,
                entity,
                ..
            } => {
                let delta = pointer_pos - start_pointer;
                let mut transform = before;
                let mut position = before.position + Vec2::new(delta.x / zoom, -delta.y / zoom);
                if self.snap_enabled {
                    position = snap_position(position, grid_world_step);
                }
                transform.position = position;
                self.app
                    .execute_command(EditorCommand::move_node(entity, transform))
            }
            ViewportDrag::Scale(drag) => self.apply_viewport_resize(drag, pointer_pos, zoom),
        };

        if let Err(error) = result {
            self.set_error(format!("{error}"));
        }
    }

    fn apply_viewport_resize(
        &mut self,
        drag: ViewportScaleDrag,
        pointer_pos: egui::Pos2,
        zoom: f32,
    ) -> Result<EditorCommandResult, EditorCommandError> {
        let delta = pointer_pos - drag.start_pointer;
        let size = resize_screen_size(drag.start_size, delta, drag.handle);

        match drag.subject {
            ViewportResizeState::Transform => {
                let mut transform = drag.before;
                let mut scale_x = drag.before.scale.x * (size.x / drag.start_size.x.max(1.0));
                let mut scale_y = drag.before.scale.y * (size.y / drag.start_size.y.max(1.0));
                if self.snap_enabled {
                    scale_x = snap_scalar(scale_x, 0.05);
                    scale_y = snap_scalar(scale_y, 0.05);
                }
                transform.scale = Vec2::new(scale_x.max(0.05), scale_y.max(0.05));
                self.app
                    .execute_command(EditorCommand::move_node(drag.entity, transform))
            }
            ViewportResizeState::Collider { .. } => {
                let Some(mut collider) = self.app.node_collider(drag.entity).copied() else {
                    return Ok(EditorCommandResult::None);
                };
                let scale_x = drag.before.scale.x.abs().max(0.0001);
                let scale_y = drag.before.scale.y.abs().max(0.0001);
                let world_width = size.x / zoom.max(0.0001);
                let world_height = size.y / zoom.max(0.0001);
                collider.half_extents = Vec2::new(
                    (world_width / (2.0 * scale_x)).max(0.5),
                    (world_height / (2.0 * scale_y)).max(0.5),
                );
                self.app
                    .execute_command(EditorCommand::attach_collider(drag.entity, collider))
            }
            ViewportResizeState::Camera { .. } => {
                let Some(mut camera) = self.app.node_camera(drag.entity).copied() else {
                    return Ok(EditorCommandResult::None);
                };
                let viewport_zoom = zoom.max(0.0001);
                let width_zoom = 640.0 * viewport_zoom / size.x.max(1.0);
                let height_zoom = 360.0 * viewport_zoom / size.y.max(1.0);
                camera.zoom = ((width_zoom + height_zoom) * 0.5).clamp(0.05, 64.0);
                self.app
                    .execute_command(EditorCommand::attach_camera(drag.entity, camera))
            }
        }
    }

    fn tilemap_layer_names(&self) -> Vec<String> {
        self.app
            .first_tilemap_node()
            .and_then(|entity| self.app.node_tilemap(entity))
            .map(|tilemap| {
                tilemap
                    .layers
                    .iter()
                    .map(|layer| layer.name.clone())
                    .collect()
            })
            .unwrap_or_else(|| vec![self.active_layer.clone()])
    }
}

fn snap_position(position: Vec2, step: f32) -> Vec2 {
    Vec2::new(snap_scalar(position.x, step), snap_scalar(position.y, step))
}

fn snap_scalar(value: f32, step: f32) -> f32 {
    if step <= 0.0 {
        return value;
    }
    (value / step).round() * step
}

fn resize_screen_size(
    start_size: egui::Vec2,
    delta: egui::Vec2,
    handle: ResizeHandle,
) -> egui::Vec2 {
    let min_size = 6.0;
    egui::vec2(
        (start_size.x + delta.x * handle.horizontal).max(min_size),
        (start_size.y + delta.y * handle.vertical).max(min_size),
    )
}
