use super::*;

pub(super) fn drag_value(
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

pub(super) fn draw_world_grid(
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

pub(super) fn viewport_grid_step(viewport_zoom: f32) -> (f32, f32) {
    let mut world_step = 32.0;
    let mut screen_step = world_step * viewport_zoom.max(0.01);

    while screen_step < 18.0 {
        world_step *= 2.0;
        screen_step *= 2.0;
    }

    while screen_step > 96.0 && world_step > 4.0 {
        world_step *= 0.5;
        screen_step *= 0.5;
    }

    (world_step, screen_step)
}

pub(super) fn draw_world_axes(painter: &egui::Painter, rect: egui::Rect, origin: egui::Pos2) {
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

pub(super) fn draw_camera_frame(
    painter: &egui::Painter,
    viewport_rect: egui::Rect,
    world_to_screen: &dyn Fn(Vec2) -> egui::Pos2,
    viewport_zoom: f32,
    item: &NodeView,
) {
    let Some(camera) = item.camera else {
        return;
    };
    let theme = theme();
    let zoom = camera.zoom.max(0.1);
    let size = egui::vec2(640.0 / zoom, 360.0 / zoom) * viewport_zoom;
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

pub(super) fn draw_collider_overlay(
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

pub(super) fn draw_node_marker(
    painter: &egui::Painter,
    center: egui::Pos2,
    color: egui::Color32,
    size: f32,
) {
    let theme = theme();
    let node_rect = egui::Rect::from_center_size(center, egui::vec2(size, size));
    painter.rect_filled(node_rect, theme.radius.sm, color.gamma_multiply(0.42));
    painter.rect_stroke(
        node_rect,
        theme.radius.sm,
        egui::Stroke::new(1.5, color),
        egui::StrokeKind::Inside,
    );
    painter.circle_filled(center, 3.0, color);
}

pub(super) fn draw_missing_texture_marker(painter: &egui::Painter, rect: egui::Rect, label: &str) {
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

pub(super) fn draw_selection_handles(painter: &egui::Painter, rect: egui::Rect) {
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

pub(super) fn draw_corner_badge(painter: &egui::Painter, position: egui::Pos2, label: &str) {
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

pub(super) fn draw_node_label(painter: &egui::Painter, position: egui::Pos2, label: &str) {
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

pub(super) fn draw_fallback_logo(painter: &egui::Painter, rect: egui::Rect) {
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

pub(super) fn fit_rect(image_size: egui::Vec2, container: egui::Rect) -> egui::Rect {
    if image_size.x <= 0.0 || image_size.y <= 0.0 {
        return container;
    }

    let scale = (container.width() / image_size.x).min(container.height() / image_size.y);
    egui::Rect::from_center_size(container.center(), image_size * scale)
}

pub(super) fn tile_uv(tile_index: u32, columns: u32, rows: u32) -> egui::Rect {
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
