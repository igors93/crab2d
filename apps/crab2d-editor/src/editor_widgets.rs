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
                .size(10.0)
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
            .min_size(egui::vec2(70.0, 28.0)),
    )
}

pub fn inspector_section(
    ui: &mut egui::Ui,
    title: &str,
    default_open: bool,
    contents: impl FnOnce(&mut egui::Ui),
) {
    let theme = theme();
    ui.add_space(theme.spacing.sm);
    egui::CollapsingHeader::new(
        egui::RichText::new(title)
            .strong()
            .size(13.0)
            .color(theme.colors.text),
    )
    .default_open(default_open)
    .show(ui, |ui| {
        inset_frame().show(ui, contents);
    });
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
            .size(11.0)
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
