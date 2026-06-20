use eframe::egui;

use crate::editor_theme::{theme, EditorTheme};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusTone {
    Info,
    Success,
    Warning,
    Error,
}

pub fn panel_frame() -> egui::Frame {
    let theme = theme();
    egui::Frame::new()
        .fill(theme.colors.panel_bg)
        .stroke(egui::Stroke::new(1.0, theme.colors.border))
        .inner_margin(egui::Margin::same(theme.spacing.panel))
}

pub fn inset_frame() -> egui::Frame {
    let theme = theme();
    egui::Frame::new()
        .fill(theme.colors.panel_bg_alt)
        .stroke(egui::Stroke::new(1.0, theme.colors.border))
        .corner_radius(theme.radius.sm)
        .inner_margin(egui::Margin::same(theme.spacing.section))
}

pub fn panel_header(ui: &mut egui::Ui, title: &str, subtitle: Option<&str>) {
    let theme = theme();
    egui::Frame::new()
        .fill(theme.colors.panel_header_bg)
        .stroke(egui::Stroke::new(1.0, theme.colors.border))
        .inner_margin(egui::Margin::symmetric(theme.spacing.panel, 8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(title)
                        .strong()
                        .size(12.0)
                        .color(theme.colors.text),
                );
                if let Some(subtitle) = subtitle {
                    ui.add_space(theme.spacing.xs);
                    chip(ui, subtitle, StatusTone::Info);
                }
            });
        });
}

pub fn toolbar_group(ui: &mut egui::Ui, label: &str, contents: impl FnOnce(&mut egui::Ui)) {
    let theme = theme();
    ui.vertical(|ui| {
        ui.label(
            egui::RichText::new(label)
                .size(9.5)
                .strong()
                .color(theme.colors.text_muted),
        );
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = theme.spacing.xs;
            contents(ui);
        });
    });
}

pub fn toolbar_button(
    ui: &mut egui::Ui,
    label: &str,
    tooltip: &str,
    enabled: bool,
    active: bool,
) -> egui::Response {
    let theme = theme();
    let fill = if active {
        theme.colors.control_active
    } else {
        theme.colors.control_bg
    };
    let stroke = if active {
        egui::Stroke::new(1.0, theme.colors.accent)
    } else {
        egui::Stroke::new(1.0, theme.colors.border)
    };
    let button = egui::Button::new(label)
        .min_size(egui::vec2(46.0, theme.sizing.toolbar_button_height))
        .fill(fill)
        .stroke(stroke)
        .corner_radius(theme.radius.sm);

    ui.add_enabled(enabled, button).on_hover_text(tooltip)
}

/// Green play button, visually prominent.
pub fn play_button(ui: &mut egui::Ui, label: &str, enabled: bool) -> egui::Response {
    let theme = theme();
    let text_color = egui::Color32::from_rgb(10, 28, 12);
    ui.add_enabled(
        enabled,
        egui::Button::new(
            egui::RichText::new(label)
                .strong()
                .color(text_color)
                .size(13.0),
        )
        .min_size(egui::vec2(66.0, theme.sizing.toolbar_button_height + 2.0))
        .fill(theme.colors.play)
        .stroke(egui::Stroke::new(1.5, theme.colors.success))
        .corner_radius(theme.radius.sm),
    )
    .on_hover_text("Run current project in Crab2D Runtime")
}

/// Full-width "Add Component" button at the bottom of the inspector.
pub fn add_component_button(ui: &mut egui::Ui) -> egui::Response {
    let theme = theme();
    let width = (ui.available_width() - 2.0).max(120.0);
    ui.add(
        egui::Button::new(
            egui::RichText::new("+ Add Component")
                .color(theme.colors.accent)
                .size(13.0),
        )
        .min_size(egui::vec2(width, 34.0))
        .fill(egui::Color32::TRANSPARENT)
        .stroke(egui::Stroke::new(1.0, theme.colors.border_strong))
        .corner_radius(theme.radius.md),
    )
    .on_hover_text("Add a component to this node")
}

pub fn icon_button(ui: &mut egui::Ui, label: &str, tooltip: &str, enabled: bool) -> egui::Response {
    let theme = theme();
    let button = egui::Button::new(label)
        .min_size(egui::vec2(
            theme.sizing.icon_button_size,
            theme.sizing.icon_button_size,
        ))
        .corner_radius(theme.radius.sm);

    ui.add_enabled(enabled, button).on_hover_text(tooltip)
}

pub fn segment_button(ui: &mut egui::Ui, label: &str, selected: bool) -> egui::Response {
    let theme = theme();
    let fill = if selected {
        theme.colors.control_active
    } else {
        theme.colors.panel_bg_alt
    };
    ui.add(
        egui::Button::selectable(selected, label)
            .fill(fill)
            .stroke(egui::Stroke::new(
                1.0,
                if selected {
                    theme.colors.accent
                } else {
                    theme.colors.border
                },
            ))
            .corner_radius(theme.radius.sm)
            .min_size(egui::vec2(72.0, 28.0)),
    )
}

/// Inspector section with a custom clickable header (chevron + title + options dots).
/// Open/closed state is persisted in egui temporary data keyed by `title`.
pub fn inspector_section(
    ui: &mut egui::Ui,
    title: &str,
    default_open: bool,
    contents: impl FnOnce(&mut egui::Ui),
) {
    let theme = theme();
    ui.add_space(theme.spacing.xs);

    let section_id = ui.id().with(("inspector_section", title));
    let is_open = ui
        .ctx()
        .data(|d| d.get_temp::<bool>(section_id).unwrap_or(default_open));

    // — Custom header row —
    let available_width = ui.available_width();
    let (header_rect, header_resp) =
        ui.allocate_exact_size(egui::vec2(available_width, 30.0), egui::Sense::click());

    if ui.is_rect_visible(header_rect) {
        let painter = ui.painter_at(header_rect);
        let fill = if header_resp.hovered() {
            theme.colors.control_hover
        } else {
            theme.colors.panel_header_bg
        };
        let corner_r = if is_open {
            egui::CornerRadius {
                nw: theme.radius.sm,
                ne: theme.radius.sm,
                sw: 0,
                se: 0,
            }
        } else {
            egui::CornerRadius::from(theme.radius.sm)
        };

        painter.rect_filled(header_rect, corner_r, fill);
        painter.rect_stroke(
            header_rect,
            corner_r,
            egui::Stroke::new(1.0, theme.colors.border),
            egui::StrokeKind::Inside,
        );

        let chevron = if is_open { "▾" } else { "▸" };
        painter.text(
            header_rect.left_center() + egui::vec2(9.0, 0.0),
            egui::Align2::LEFT_CENTER,
            chevron,
            egui::FontId::proportional(12.0),
            theme.colors.text_muted,
        );
        painter.text(
            header_rect.left_center() + egui::vec2(24.0, 0.0),
            egui::Align2::LEFT_CENTER,
            title,
            egui::FontId::proportional(12.5),
            theme.colors.text,
        );
        painter.text(
            header_rect.right_center() - egui::vec2(10.0, 0.0),
            egui::Align2::RIGHT_CENTER,
            "···",
            egui::FontId::proportional(12.0),
            theme.colors.text_muted,
        );
    }

    if header_resp.clicked() {
        ui.ctx().data_mut(|d| d.insert_temp(section_id, !is_open));
    }

    if is_open {
        let bottom_radius = egui::CornerRadius {
            nw: 0,
            ne: 0,
            sw: theme.radius.sm,
            se: theme.radius.sm,
        };
        egui::Frame::new()
            .fill(theme.colors.panel_bg_alt)
            .stroke(egui::Stroke::new(1.0, theme.colors.border))
            .corner_radius(bottom_radius)
            .inner_margin(egui::Margin::same(theme.spacing.section))
            .show(ui, contents);
    }
}

pub fn property_row(ui: &mut egui::Ui, label: &str, contents: impl FnOnce(&mut egui::Ui)) {
    let theme = theme();
    ui.horizontal(|ui| {
        ui.add_sized(
            [theme.sizing.property_label_width, 22.0],
            egui::Label::new(egui::RichText::new(label).color(theme.colors.text_secondary)),
        );
        contents(ui);
    });
}

pub fn chip(ui: &mut egui::Ui, text: &str, tone: StatusTone) -> egui::Response {
    let theme = theme();
    let color = tone_color(theme, tone);
    egui::Frame::new()
        .fill(tint(color, 32))
        .stroke(egui::Stroke::new(1.0, tint(color, 150)))
        .corner_radius(theme.radius.xs)
        .inner_margin(egui::Margin::symmetric(6, 2))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(text).size(10.5).strong().color(color))
        })
        .inner
}

pub fn status_badge(ui: &mut egui::Ui, text: &str, tone: StatusTone) {
    let theme = theme();
    let color = tone_color(theme, tone);
    egui::Frame::new()
        .fill(tint(color, 24))
        .stroke(egui::Stroke::new(1.0, tint(color, 140)))
        .corner_radius(theme.radius.sm)
        .inner_margin(egui::Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(text)
                    .size(12.0)
                    .color(theme.colors.text),
            );
        });
}

pub fn section_label(ui: &mut egui::Ui, text: &str) {
    let theme = theme();
    ui.label(
        egui::RichText::new(text)
            .size(10.5)
            .strong()
            .color(theme.colors.text_muted),
    );
}

fn tone_color(theme: EditorTheme, tone: StatusTone) -> egui::Color32 {
    match tone {
        StatusTone::Info => theme.colors.accent,
        StatusTone::Success => theme.colors.success,
        StatusTone::Warning => theme.colors.warning,
        StatusTone::Error => theme.colors.error,
    }
}

fn tint(color: egui::Color32, alpha: u8) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}
