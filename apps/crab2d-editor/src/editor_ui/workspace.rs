use super::*;

impl Crab2DEditorUi {
    pub(super) fn show_workspace(&mut self, root: &mut egui::Ui) {
        self.show_workspace_slot(root, DockSlot::Left);
        self.show_workspace_slot(root, DockSlot::Right);
        self.show_workspace_slot(root, DockSlot::Bottom);
        self.show_viewport(root);
    }

    pub(super) fn show_scene_tabs(&mut self, root: &mut egui::Ui) {
        let theme = theme();
        egui::Panel::top("scene_tabs")
            .exact_size(34.0)
            .frame(
                egui::Frame::new()
                    .fill(theme.colors.panel_bg_alt)
                    .stroke(egui::Stroke::new(1.0, theme.colors.border))
                    .inner_margin(egui::Margin::symmetric(10, 4)),
            )
            .show_inside(root, |ui| {
                ui.horizontal_centered(|ui| {
                    let title = "MainScene.tscn";

                    let scene_selected = self.left_panel_tab == LeftPanelTab::Scene;
                    if widgets::segment_button(ui, title, scene_selected).clicked() {
                        self.left_panel_tab = LeftPanelTab::Scene;
                        self.scene_panel_visible = true;
                        if self.scene_panel_slot == DockSlot::Hidden {
                            self.scene_panel_slot = DockSlot::Left;
                        }
                    }

                    if let Some(selected) = self.selected {
                        let node_name = self
                            .app
                            .find_node(selected)
                            .map(|node| node.name.clone())
                            .unwrap_or_else(|| "Node".to_owned());
                        if widgets::toolbar_button(
                            ui,
                            format!("{}  x", truncate_text(&node_name, 24)).as_str(),
                            "Selected node",
                            true,
                            false,
                        )
                        .clicked()
                        {
                            self.inspector_visible = true;
                            if self.inspector_slot == DockSlot::Hidden {
                                self.inspector_slot = DockSlot::Right;
                            }
                        }
                    }

                    if widgets::icon_button(ui, "+", "Create node", true).clicked() {
                        self.create_node();
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        self.show_workspace_menu(ui);
                    });
                });
            });
    }

    pub(super) fn show_status_bar(&mut self, root: &mut egui::Ui) {
        let theme = theme();
        egui::Panel::bottom("status_bar")
            .exact_size(24.0)
            .frame(
                egui::Frame::new()
                    .fill(theme.colors.panel_bg)
                    .stroke(egui::Stroke::new(1.0, theme.colors.border))
                    .inner_margin(egui::Margin::symmetric(10, 2)),
            )
            .show_inside(root, |ui| {
                ui.horizontal_centered(|ui| {
                    widgets::status_badge(ui, &self.status, self.status_tone);
                    ui.separator();
                    ui.label(
                        egui::RichText::new(self.selected_summary())
                            .size(11.0)
                            .color(theme.colors.text_secondary),
                    );
                    ui.separator();
                    ui.label(
                        egui::RichText::new(format!("Layer: {}", self.active_layer))
                            .size(11.0)
                            .color(theme.colors.text_secondary),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new("60 FPS")
                                .size(11.0)
                                .color(theme.colors.text_muted),
                        );
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!(
                                "Zoom: {:.0}%",
                                self.viewport_zoom * 100.0
                            ))
                            .size(11.0)
                            .color(theme.colors.text_muted),
                        );
                        ui.separator();
                        widgets::chip(
                            ui,
                            if self.snap_enabled {
                                "Snap ON"
                            } else {
                                "Snap OFF"
                            },
                            StatusTone::Info,
                        );
                        widgets::chip(
                            ui,
                            if self.show_grid {
                                "Grid ON"
                            } else {
                                "Grid OFF"
                            },
                            StatusTone::Info,
                        );
                    });
                });
            });
    }

    pub(super) fn show_workspace_menu(&mut self, ui: &mut egui::Ui) {
        let theme = theme();
        ui.menu_button("Workspace", |ui| {
            ui.set_min_width(340.0);
            ui.label(
                egui::RichText::new("Arrange panels")
                    .strong()
                    .color(theme.colors.text),
            );
            ui.add_space(theme.spacing.sm);

            for panel in DockPanel::ALL {
                widgets::section_label(ui, panel.label());
                ui.horizontal_wrapped(|ui| {
                    for slot in DockSlot::ALL {
                        let selected = self.panel_slot(panel) == slot && self.panel_visible(panel);
                        if widgets::segment_button(ui, slot.label(), selected).clicked() {
                            self.set_panel_slot(panel, slot);
                        }
                    }
                });
                ui.add_space(theme.spacing.sm);
            }

            ui.separator();
            if ui.button("Reset layout").clicked() {
                self.reset_workspace_layout();
                ui.close();
            }
        });
    }

    fn show_workspace_slot(&mut self, root: &mut egui::Ui, slot: DockSlot) {
        let panels: Vec<DockPanel> = DockPanel::ALL
            .into_iter()
            .filter(|panel| self.panel_visible(*panel) && self.panel_slot(*panel) == slot)
            .collect();
        if panels.is_empty() {
            return;
        }

        for panel in panels {
            match slot {
                DockSlot::Left => self.show_side_dock(root, panel, true),
                DockSlot::Right => self.show_side_dock(root, panel, false),
                DockSlot::Bottom => self.show_bottom_workspace_dock(root, panel),
                DockSlot::Hidden => {}
            }
        }
    }

    fn show_side_dock(&mut self, root: &mut egui::Ui, panel: DockPanel, left: bool) {
        let theme = theme();
        let id = egui::Id::new(("workspace_side", panel, left));
        let default_width = match panel {
            DockPanel::Scene => theme.sizing.left_panel_width,
            DockPanel::Inspector => theme.sizing.inspector_width,
            DockPanel::Assets => 340.0,
        };
        let min_width = match panel {
            DockPanel::Assets => 260.0,
            _ => 220.0,
        };

        let panel_builder = if left {
            egui::Panel::left(id)
        } else {
            egui::Panel::right(id)
        };

        panel_builder
            .resizable(true)
            .default_size(default_width)
            .min_size(min_width)
            .max_size(560.0)
            .frame(widgets::panel_frame())
            .show_inside(root, |ui| {
                self.show_dock_panel_contents(ui, panel);
            });
    }

    fn show_bottom_workspace_dock(&mut self, root: &mut egui::Ui, panel: DockPanel) {
        let theme = theme();
        let default_height = match panel {
            DockPanel::Assets => theme.sizing.bottom_dock_height,
            DockPanel::Scene => 260.0,
            DockPanel::Inspector => 300.0,
        };
        let min_height = if panel == DockPanel::Assets && self.asset_dock_collapsed {
            34.0
        } else {
            148.0
        };

        egui::Panel::bottom(egui::Id::new(("workspace_bottom", panel)))
            .resizable(true)
            .default_size(default_height)
            .min_size(min_height)
            .frame(widgets::panel_frame())
            .show_inside(root, |ui| {
                self.show_dock_panel_contents(ui, panel);
            });
    }

    fn show_dock_panel_contents(&mut self, ui: &mut egui::Ui, panel: DockPanel) {
        self.show_dock_shell(ui, panel, |editor, ui| match panel {
            DockPanel::Scene => editor.show_scene_panel_contents(ui),
            DockPanel::Inspector => editor.show_inspector_contents(ui),
            DockPanel::Assets => editor.show_asset_dock_contents(ui),
        });
    }

    fn show_dock_shell(
        &mut self,
        ui: &mut egui::Ui,
        panel: DockPanel,
        contents: impl FnOnce(&mut Self, &mut egui::Ui),
    ) {
        let theme = theme();
        ui.horizontal_centered(|ui| {
            ui.label(
                egui::RichText::new(panel.label())
                    .strong()
                    .size(11.5)
                    .color(theme.colors.text_secondary),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if widgets::icon_button(ui, "x", "Hide panel", true).clicked() {
                    self.set_panel_slot(panel, DockSlot::Hidden);
                }
                ui.menu_button("Dock", |ui| {
                    for slot in DockSlot::ALL {
                        if ui.button(slot.label()).clicked() {
                            self.set_panel_slot(panel, slot);
                            ui.close();
                        }
                    }
                });
            });
        });
        ui.add_space(theme.spacing.xs);
        contents(self, ui);
    }

    fn selected_summary(&self) -> String {
        self.selected
            .and_then(|id| self.app.find_node(id))
            .map(|node| format!("Selected: {}", node.name))
            .unwrap_or_else(|| "No selection".to_owned())
    }

    fn panel_slot(&self, panel: DockPanel) -> DockSlot {
        match panel {
            DockPanel::Scene => self.scene_panel_slot,
            DockPanel::Inspector => self.inspector_slot,
            DockPanel::Assets => self.asset_dock_slot,
        }
    }

    fn panel_visible(&self, panel: DockPanel) -> bool {
        match panel {
            DockPanel::Scene => self.scene_panel_visible,
            DockPanel::Inspector => self.inspector_visible,
            DockPanel::Assets => self.asset_dock_visible,
        }
    }

    fn set_panel_slot(&mut self, panel: DockPanel, slot: DockSlot) {
        match panel {
            DockPanel::Scene => {
                self.scene_panel_slot = slot;
                self.scene_panel_visible = slot != DockSlot::Hidden;
            }
            DockPanel::Inspector => {
                self.inspector_slot = slot;
                self.inspector_visible = slot != DockSlot::Hidden;
            }
            DockPanel::Assets => {
                self.asset_dock_slot = slot;
                self.asset_dock_visible = slot != DockSlot::Hidden;
            }
        }
        self.set_status(format!("{} docked: {}", panel.label(), slot.label()));
    }

    fn reset_workspace_layout(&mut self) {
        self.scene_panel_slot = DockSlot::Left;
        self.inspector_slot = DockSlot::Right;
        self.asset_dock_slot = DockSlot::Bottom;
        self.scene_panel_visible = true;
        self.inspector_visible = true;
        self.asset_dock_visible = true;
        self.asset_dock_collapsed = false;
        self.set_success("Workspace layout reset");
    }
}
