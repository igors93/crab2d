use eframe::egui;

#[derive(Debug, Clone, Copy)]
pub struct EditorTheme {
    pub colors: EditorColors,
    pub spacing: EditorSpacing,
    pub sizing: EditorSizing,
    pub radius: EditorRadius,
}

#[derive(Debug, Clone, Copy)]
pub struct EditorColors {
    pub app_bg: egui::Color32,
    pub panel_bg: egui::Color32,
    pub panel_bg_alt: egui::Color32,
    pub panel_header_bg: egui::Color32,
    pub viewport_bg: egui::Color32,
    pub viewport_overlay: egui::Color32,
    pub control_bg: egui::Color32,
    pub control_hover: egui::Color32,
    pub control_active: egui::Color32,
    pub border: egui::Color32,
    pub border_strong: egui::Color32,
    pub text: egui::Color32,
    pub text_secondary: egui::Color32,
    pub text_muted: egui::Color32,
    pub accent: egui::Color32,
    pub accent_soft: egui::Color32,
    pub success: egui::Color32,
    pub warning: egui::Color32,
    pub error: egui::Color32,
    pub error_bg: egui::Color32,
    pub grid_minor: egui::Color32,
    pub grid_major: egui::Color32,
    pub axis_x: egui::Color32,
    pub axis_y: egui::Color32,
    pub camera: egui::Color32,
}

#[derive(Debug, Clone, Copy)]
pub struct EditorSpacing {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub panel: i8,
    pub section: i8,
}

#[derive(Debug, Clone, Copy)]
pub struct EditorSizing {
    pub top_bar_height: f32,
    pub left_panel_width: f32,
    pub inspector_width: f32,
    pub bottom_dock_height: f32,
    pub toolbar_button_height: f32,
    pub icon_button_size: f32,
    pub asset_card_width: f32,
    pub asset_card_height: f32,
    pub tile_button_size: f32,
    pub property_label_width: f32,
    pub property_input_width: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct EditorRadius {
    pub xs: u8,
    pub sm: u8,
    pub md: u8,
}

pub fn theme() -> EditorTheme {
    EditorTheme {
        colors: EditorColors {
            app_bg: egui::Color32::from_rgb(12, 16, 19),
            panel_bg: egui::Color32::from_rgb(22, 28, 32),
            panel_bg_alt: egui::Color32::from_rgb(27, 34, 39),
            panel_header_bg: egui::Color32::from_rgb(31, 39, 45),
            viewport_bg: egui::Color32::from_rgb(15, 19, 22),
            viewport_overlay: egui::Color32::from_rgba_unmultiplied(18, 24, 28, 226),
            control_bg: egui::Color32::from_rgb(33, 42, 48),
            control_hover: egui::Color32::from_rgb(43, 55, 62),
            control_active: egui::Color32::from_rgb(26, 82, 86),
            border: egui::Color32::from_rgb(52, 66, 74),
            border_strong: egui::Color32::from_rgb(75, 94, 104),
            text: egui::Color32::from_rgb(230, 237, 240),
            text_secondary: egui::Color32::from_rgb(176, 190, 196),
            text_muted: egui::Color32::from_rgb(119, 137, 145),
            accent: egui::Color32::from_rgb(42, 206, 199),
            accent_soft: egui::Color32::from_rgb(95, 231, 224),
            success: egui::Color32::from_rgb(116, 211, 133),
            warning: egui::Color32::from_rgb(231, 178, 87),
            error: egui::Color32::from_rgb(245, 112, 112),
            error_bg: egui::Color32::from_rgb(78, 42, 46),
            grid_minor: egui::Color32::from_rgba_unmultiplied(91, 115, 122, 42),
            grid_major: egui::Color32::from_rgba_unmultiplied(105, 163, 166, 78),
            axis_x: egui::Color32::from_rgba_unmultiplied(238, 101, 101, 120),
            axis_y: egui::Color32::from_rgba_unmultiplied(105, 211, 125, 120),
            camera: egui::Color32::from_rgb(135, 157, 255),
        },
        spacing: EditorSpacing {
            xs: 4.0,
            sm: 6.0,
            md: 10.0,
            lg: 14.0,
            panel: 10,
            section: 8,
        },
        sizing: EditorSizing {
            top_bar_height: 64.0,
            left_panel_width: 270.0,
            inspector_width: 360.0,
            bottom_dock_height: 225.0,
            toolbar_button_height: 28.0,
            icon_button_size: 28.0,
            asset_card_width: 142.0,
            asset_card_height: 132.0,
            tile_button_size: 34.0,
            property_label_width: 92.0,
            property_input_width: 92.0,
        },
        radius: EditorRadius {
            xs: 3,
            sm: 5,
            md: 7,
        },
    }
}

pub fn configure_style(ctx: &egui::Context) {
    let theme = theme();
    let mut style = (*ctx.global_style()).clone();

    style.visuals = egui::Visuals::dark();
    style.visuals.window_fill = theme.colors.app_bg;
    style.visuals.panel_fill = theme.colors.app_bg;
    style.visuals.faint_bg_color = theme.colors.panel_bg_alt;
    style.visuals.extreme_bg_color = theme.colors.app_bg;
    style.visuals.hyperlink_color = theme.colors.accent_soft;
    style.visuals.selection.bg_fill = theme.colors.control_active;
    style.visuals.selection.stroke = egui::Stroke::new(1.0, theme.colors.accent);
    style.visuals.widgets.noninteractive.bg_fill = theme.colors.panel_bg;
    style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, theme.colors.text);
    style.visuals.widgets.inactive.bg_fill = theme.colors.control_bg;
    style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, theme.colors.text);
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, theme.colors.border);
    style.visuals.widgets.hovered.bg_fill = theme.colors.control_hover;
    style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, theme.colors.text);
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, theme.colors.border_strong);
    style.visuals.widgets.active.bg_fill = theme.colors.control_active;
    style.visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, theme.colors.text);
    style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, theme.colors.accent);
    style.visuals.widgets.open.bg_fill = theme.colors.panel_bg_alt;
    style.visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, theme.colors.text);
    style.visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, theme.colors.border);

    style.spacing.item_spacing = egui::vec2(theme.spacing.sm, theme.spacing.sm);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);
    style.spacing.menu_margin = egui::Margin::same(theme.spacing.section);
    style.spacing.indent = 14.0;

    ctx.set_global_style(style);
}

pub fn tile_color(tile_index: u32) -> egui::Color32 {
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
