use super::*;

impl Crab2DEditorUi {
    pub(super) fn show_top_bar(&mut self, root: &mut egui::Ui) {
        let theme = theme();
        egui::Panel::top("top_bar")
            .exact_size(theme.sizing.top_bar_height)
            .frame(
                egui::Frame::new()
                    .fill(theme.colors.panel_bg)
                    .stroke(egui::Stroke::new(1.0, theme.colors.border))
                    .inner_margin(egui::Margin::symmetric(10, 8)),
            )
            .show_inside(root, |ui| {
                ui.horizontal_centered(|ui| {
                    self.show_logo(ui);
                    ui.add_space(6.0);
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Crab2D Editor")
                                    .strong()
                                    .size(15.0)
                                    .color(theme.colors.text),
                            );
                            ui.add_space(2.0);
                            widgets::chip(ui, "v0.2.0", widgets::StatusTone::Info);
                        });
                        ui.label(
                            egui::RichText::new(self.app.project_session().display_title())
                                .size(11.0)
                                .color(theme.colors.text_secondary),
                        );
                        let path = self
                            .app
                            .project_path()
                            .map(|path| path.display().to_string())
                            .unwrap_or_else(|| "No project file".to_owned());
                        ui.label(
                            egui::RichText::new(format!(
                                "{} | {}",
                                self.app.project_session().status_label(),
                                truncate_text(&path, 40)
                            ))
                            .size(10.0)
                            .color(theme.colors.text_muted),
                        );
                    });

                    ui.add_space(theme.spacing.lg);
                    widgets::toolbar_group(ui, "PROJECT", |ui| {
                        if widgets::toolbar_button(ui, "New", "Create a new project", true, false)
                            .clicked()
                        {
                            self.new_project();
                        }
                        if widgets::toolbar_button(ui, "Open", "Open project file", true, false)
                            .clicked()
                        {
                            self.open_project_dialog();
                        }
                        if widgets::toolbar_button(ui, "Save", "Save project", true, false)
                            .clicked()
                        {
                            self.save_project();
                        }
                        if widgets::toolbar_button(ui, "Save As", "Save project as", true, false)
                            .clicked()
                        {
                            self.save_as_dialog();
                        }
                        let can_reload = self.app.project_path().is_some();
                        if widgets::toolbar_button(
                            ui,
                            "Reload",
                            "Reload scene from saved file",
                            can_reload,
                            false,
                        )
                        .clicked()
                        {
                            self.reload_scene_from_disk();
                        }
                    });
                    ui.separator();

                    widgets::toolbar_group(ui, "EDIT", |ui| {
                        if widgets::toolbar_button(
                            ui,
                            "Undo",
                            "Undo last command",
                            self.app.can_undo(),
                            false,
                        )
                        .clicked()
                        {
                            self.undo();
                        }
                        if widgets::toolbar_button(
                            ui,
                            "Redo",
                            "Redo last command",
                            self.app.can_redo(),
                            false,
                        )
                        .clicked()
                        {
                            self.redo();
                        }
                    });
                    ui.separator();

                    widgets::toolbar_group(ui, "RUN", |ui| {
                        if widgets::play_button(ui, "▶ Play", true).clicked() {
                            self.play_project();
                        }
                        widgets::toolbar_button(ui, "⏸ Pause", "Pause preview", false, false);
                        widgets::toolbar_button(ui, "⏹ Stop", "Stop preview", false, false);
                    });
                    ui.separator();

                    widgets::toolbar_group(ui, "TOOLS", |ui| {
                        self.show_tool_button(ui, EditorTool::Select, "Select", "Select nodes");
                        self.show_tool_button(ui, EditorTool::Pan, "Pan", "Pan the viewport");
                        self.show_tool_button(
                            ui,
                            EditorTool::TileBrush,
                            "Brush",
                            "Paint selected tile",
                        );
                        self.show_tool_button(ui, EditorTool::EraseTile, "Erase", "Erase tiles");
                    });
                    ui.separator();

                    widgets::toolbar_group(ui, "WORKSPACE", |ui| {
                        self.show_workspace_menu(ui);
                    });
                    ui.separator();

                    widgets::toolbar_group(ui, "BUILD", |ui| {
                        widgets::toolbar_button(ui, "Export", "Build and export", false, false);
                    });
                    widgets::toolbar_group(ui, "PLUGINS", |ui| {
                        widgets::toolbar_button(ui, "Market", "Plugin market", false, false);
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        widgets::chip(ui, "Rust", StatusTone::Info);
                        ui.label(
                            egui::RichText::new("FPS: 60")
                                .size(10.0)
                                .color(theme.colors.text_muted),
                        );
                    });
                });
            });
    }

    pub(super) fn show_tool_button(
        &mut self,
        ui: &mut egui::Ui,
        tool: EditorTool,
        label: &str,
        tooltip: &str,
    ) {
        if widgets::toolbar_button(ui, label, tooltip, true, self.active_tool == tool).clicked() {
            self.active_tool = tool;
            self.set_status(format!("{label} tool active"));
        }
    }
    pub(super) fn show_logo(&mut self, ui: &mut egui::Ui) {
        let theme = theme();
        let (rect, response) = ui.allocate_exact_size(egui::vec2(38.0, 38.0), egui::Sense::hover());
        match self.textures.load(ui.ctx(), "logo.png") {
            TextureLookup::Loaded(texture) => {
                ui.painter().rect_filled(
                    rect.expand(2.0),
                    theme.radius.md,
                    theme.colors.panel_bg_alt,
                );
                ui.painter().image(
                    texture.id(),
                    rect,
                    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }
            TextureLookup::Failed(error) => {
                draw_fallback_logo(ui.painter(), rect);
                response.on_hover_text(error);
            }
            TextureLookup::Missing => {
                draw_fallback_logo(ui.painter(), rect);
            }
        }
    }
}
