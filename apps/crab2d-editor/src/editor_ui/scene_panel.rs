use super::*;

impl Crab2DEditorUi {
    pub(super) fn show_scene_panel_contents(&mut self, ui: &mut egui::Ui) {
        let theme = theme();
        widgets::panel_header(ui, "SCENE", Some("2D"));
        ui.add_space(theme.spacing.xs);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = theme.spacing.xs;
            if widgets::segment_button(ui, "Scene", self.left_panel_tab == LeftPanelTab::Scene)
                .clicked()
            {
                self.left_panel_tab = LeftPanelTab::Scene;
            }
            if widgets::segment_button(ui, "Layers", self.left_panel_tab == LeftPanelTab::Layers)
                .clicked()
            {
                self.left_panel_tab = LeftPanelTab::Layers;
            }
            if widgets::segment_button(ui, "Library", self.left_panel_tab == LeftPanelTab::Library)
                .clicked()
            {
                self.left_panel_tab = LeftPanelTab::Library;
            }
        });
        ui.add_space(theme.spacing.sm);

        match self.left_panel_tab {
            LeftPanelTab::Scene => self.show_scene_hierarchy(ui),
            LeftPanelTab::Layers => self.show_layers_panel(ui),
            LeftPanelTab::Library => self.show_library_panel(ui),
        }
    }

    pub(super) fn show_scene_hierarchy(&mut self, ui: &mut egui::Ui) {
        let theme = theme();
        ui.horizontal(|ui| {
            let search_width = (ui.available_width() - 70.0).max(80.0);
            ui.add_sized(
                [search_width, 26.0],
                egui::TextEdit::singleline(&mut self.scene_filter_edit).hint_text("Filter nodes"),
            );
            ui.menu_button("+ Add", |ui| {
                if ui.button("Empty Node").clicked() {
                    self.create_node();
                    ui.close();
                }
                if ui.button("Camera2D").clicked() {
                    self.create_camera_node();
                    ui.close();
                }
                if ui.button("World Text").clicked() {
                    self.create_world_text_node();
                    ui.close();
                }
                ui.separator();
                for preset in GameplayPreset::ALL {
                    if ui.button(preset.label()).clicked() {
                        self.create_preset_node(preset);
                        ui.close();
                    }
                }
            });
        });

        ui.add_space(theme.spacing.sm);
        widgets::section_label(ui, "SCENE TREE");
        let filter = self.scene_filter_edit.to_lowercase();
        let ids: Vec<EntityId> = self
            .app
            .scene_nodes()
            .iter()
            .filter(|node| filter.is_empty() || node.name.to_lowercase().contains(filter.as_str()))
            .map(|node| node.id)
            .collect();

        widgets::inset_frame().show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("scene_tree_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    self.show_scene_group(ui, "World", &ids, |editor, id| {
                        editor.app.node_tilemap(id).is_some()
                            || editor.app.node_camera(id).is_some()
                    });
                    self.show_scene_group(ui, "Actors", &ids, |editor, id| {
                        editor.app.node_player_controller(id).is_some()
                            || (editor.app.node_sprite(id).is_some()
                                && editor.app.node_trigger(id).is_none())
                    });
                    self.show_scene_group(ui, "Gameplay", &ids, |editor, id| {
                        editor.app.node_trigger(id).is_some()
                            || editor.app.node_camera_follow(id).is_some()
                    });
                    self.show_scene_group(ui, "Other", &ids, |editor, id| {
                        editor.app.node_tilemap(id).is_none()
                            && editor.app.node_camera(id).is_none()
                            && editor.app.node_player_controller(id).is_none()
                            && editor.app.node_sprite(id).is_none()
                            && editor.app.node_trigger(id).is_none()
                    });
                });
        });

        ui.add_space(theme.spacing.md);
        widgets::section_label(ui, "WORLD");
        widgets::inset_frame().show(ui, |ui| {
            if let Some(tilemap_id) = self.app.first_tilemap_node() {
                if let Some(tilemap) = self.app.node_tilemap(tilemap_id) {
                    widgets::property_row(ui, "Map", |ui| {
                        ui.monospace(format!(
                            "{} x {}",
                            tilemap.map_size.width, tilemap.map_size.height
                        ));
                    });
                    widgets::property_row(ui, "Tile", |ui| {
                        ui.monospace(format!(
                            "{} x {}",
                            tilemap.tile_size.width, tilemap.tile_size.height
                        ));
                    });
                    widgets::property_row(ui, "Layers", |ui| {
                        ui.label(tilemap.layers.len().to_string());
                    });
                }
            } else {
                ui.label(egui::RichText::new("No tilemap").color(theme.colors.text_muted));
            }
        });
    }

    pub(super) fn show_scene_group(
        &mut self,
        ui: &mut egui::Ui,
        label: &str,
        ids: &[EntityId],
        predicate: impl Fn(&Self, EntityId) -> bool,
    ) {
        let group_ids: Vec<EntityId> = ids
            .iter()
            .copied()
            .filter(|id| predicate(self, *id))
            .collect();
        if group_ids.is_empty() {
            return;
        }

        egui::CollapsingHeader::new(format!("{label}  {}", group_ids.len()))
            .default_open(true)
            .show(ui, |ui| {
                for id in group_ids {
                    self.show_scene_node_row(ui, id, 12.0);
                }
            });
    }

    pub(super) fn show_scene_node_row(&mut self, ui: &mut egui::Ui, id: EntityId, indent: f32) {
        let theme = theme();
        let is_selected = self.selected == Some(id);
        let label = self
            .app
            .find_node(id)
            .map(|node| node.name.clone())
            .unwrap_or_else(|| "?".to_owned());
        let (kind, tone) = self.node_kind(id);
        let node_type = self.node_type(id);

        ui.add_space(1.0);
        let available = ui.available_width();
        let row_height = 30.0;
        let (row_rect, row_resp) =
            ui.allocate_exact_size(egui::vec2(available, row_height), egui::Sense::click());

        if ui.is_rect_visible(row_rect) {
            let painter = ui.painter_at(row_rect);
            let fill = if is_selected {
                theme.colors.control_active
            } else if row_resp.hovered() {
                theme.colors.control_hover
            } else {
                egui::Color32::TRANSPARENT
            };
            let stroke_color = if is_selected {
                theme.colors.accent
            } else {
                egui::Color32::TRANSPARENT
            };
            painter.rect_filled(row_rect, theme.radius.sm, fill);
            if is_selected {
                painter.rect_stroke(
                    row_rect,
                    theme.radius.sm,
                    egui::Stroke::new(1.0, stroke_color),
                    egui::StrokeKind::Inside,
                );
                // accent left border
                let left_bar = egui::Rect::from_min_size(
                    row_rect.left_top(),
                    egui::vec2(3.0, row_rect.height()),
                );
                painter.rect_filled(
                    left_bar,
                    egui::CornerRadius {
                        nw: theme.radius.sm,
                        ne: 0,
                        sw: theme.radius.sm,
                        se: 0,
                    },
                    theme.colors.accent,
                );
            }

            // Chip badge
            let chip_text = kind;
            let chip_w = 32.0_f32;
            let chip_rect = egui::Rect::from_min_size(
                row_rect.left_top() + egui::vec2(6.0 + indent, (row_height - 16.0) / 2.0),
                egui::vec2(chip_w, 16.0),
            );
            let chip_color = match tone {
                StatusTone::Info => theme.colors.accent,
                StatusTone::Success => theme.colors.success,
                StatusTone::Warning => theme.colors.warning,
                StatusTone::Error => theme.colors.error,
            };
            painter.rect_filled(
                chip_rect,
                theme.radius.xs,
                egui::Color32::from_rgba_unmultiplied(
                    chip_color.r(),
                    chip_color.g(),
                    chip_color.b(),
                    36,
                ),
            );
            painter.rect_stroke(
                chip_rect,
                theme.radius.xs,
                egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgba_unmultiplied(
                        chip_color.r(),
                        chip_color.g(),
                        chip_color.b(),
                        140,
                    ),
                ),
                egui::StrokeKind::Inside,
            );
            painter.text(
                chip_rect.center(),
                egui::Align2::CENTER_CENTER,
                chip_text,
                egui::FontId::monospace(9.5),
                chip_color,
            );

            // Node name
            painter.text(
                row_rect.left_center() + egui::vec2(44.0 + indent, 0.0),
                egui::Align2::LEFT_CENTER,
                truncate_text(&label, 22),
                egui::FontId::proportional(12.5),
                if is_selected {
                    theme.colors.text
                } else {
                    theme.colors.text_secondary
                },
            );

            // Node type (dimmed, on the right)
            painter.text(
                row_rect.right_center() - egui::vec2(42.0, 0.0),
                egui::Align2::RIGHT_CENTER,
                node_type,
                egui::FontId::proportional(10.0),
                theme.colors.text_muted,
            );
            painter.text(
                row_rect.right_center() - egui::vec2(7.0, 0.0),
                egui::Align2::RIGHT_CENTER,
                "◉",
                egui::FontId::proportional(11.0),
                theme.colors.text_muted,
            );
        }

        if row_resp.clicked() {
            self.select_node(id);
        }
        row_resp.on_hover_text(self.node_label(id));
    }

    pub(super) fn show_layers_panel(&mut self, ui: &mut egui::Ui) {
        let theme = theme();
        widgets::section_label(ui, "LAYERS");
        widgets::inset_frame().show(ui, |ui| {
            self.show_layer_row(ui, "UI", theme.colors.warning, false);
            self.show_layer_row(ui, "Entities", theme.colors.accent, true);
            self.show_layer_row(ui, "Props", theme.colors.success, true);
            self.show_layer_row(ui, "Tilemap", theme.colors.camera, true);
            self.show_layer_row(ui, "Background", theme.colors.text_muted, true);
        });

        ui.add_space(theme.spacing.md);
        widgets::section_label(ui, "TILEMAP LAYERS");
        widgets::inset_frame().show(ui, |ui| {
            let Some(tilemap_id) = self.app.first_tilemap_node() else {
                ui.label(egui::RichText::new("No tilemap").color(theme.colors.text_muted));
                return;
            };
            let Some(tilemap) = self.app.node_tilemap(tilemap_id) else {
                return;
            };
            let layer_names: Vec<String> = tilemap
                .layers
                .iter()
                .map(|layer| layer.name.clone())
                .collect();
            for layer_name in layer_names {
                let selected = self.active_layer == layer_name;
                let response = ui.add(
                    egui::Button::selectable(selected, format!("▦ {layer_name}"))
                        .fill(if selected {
                            theme.colors.control_active
                        } else {
                            theme.colors.panel_bg_alt
                        })
                        .stroke(egui::Stroke::new(
                            1.0,
                            if selected {
                                theme.colors.accent
                            } else {
                                theme.colors.border
                            },
                        ))
                        .corner_radius(theme.radius.sm)
                        .min_size(egui::vec2(ui.available_width(), 28.0)),
                );
                if response.clicked() {
                    self.active_layer = layer_name;
                    self.set_status(format!("Layer active: {}", self.active_layer));
                }
            }
        });
    }

    fn show_layer_row(
        &mut self,
        ui: &mut egui::Ui,
        name: &str,
        color: egui::Color32,
        visible: bool,
    ) {
        let theme = theme();
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 28.0), egui::Sense::click());
        let fill = if response.hovered() {
            theme.colors.control_hover
        } else {
            egui::Color32::TRANSPARENT
        };
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, theme.radius.sm, fill);
        painter.circle_filled(rect.left_center() + egui::vec2(12.0, 0.0), 5.0, color);
        painter.text(
            rect.left_center() + egui::vec2(26.0, 0.0),
            egui::Align2::LEFT_CENTER,
            name,
            egui::FontId::proportional(12.5),
            theme.colors.text,
        );
        painter.text(
            rect.right_center() - egui::vec2(26.0, 0.0),
            egui::Align2::RIGHT_CENTER,
            if visible { "◉" } else { "○" },
            egui::FontId::proportional(11.0),
            theme.colors.text_muted,
        );
        painter.text(
            rect.right_center() - egui::vec2(7.0, 0.0),
            egui::Align2::RIGHT_CENTER,
            "▣",
            egui::FontId::proportional(11.0),
            theme.colors.text_muted,
        );
    }

    pub(super) fn show_library_panel(&mut self, ui: &mut egui::Ui) {
        let theme = theme();
        widgets::section_label(ui, "GAMEPLAY PRESETS");
        widgets::inset_frame().show(ui, |ui| {
            let card_width = ((ui.available_width() - theme.spacing.sm) / 2.0).max(112.0);
            ui.horizontal_wrapped(|ui| {
                for preset in GameplayPreset::ALL {
                    let response = ui.add_sized(
                        [card_width, 30.0],
                        egui::Button::new(preset.label())
                            .fill(theme.colors.control_bg)
                            .stroke(egui::Stroke::new(1.0, theme.colors.border))
                            .corner_radius(theme.radius.sm),
                    );
                    if response.clicked() {
                        self.create_preset_node(preset);
                    }
                }
            });
        });
        ui.add_space(theme.spacing.md);

        ui.horizontal(|ui| {
            let search_width =
                (ui.available_width() - theme.sizing.icon_button_size - 8.0).max(80.0);
            ui.add_sized(
                [search_width, 26.0],
                egui::TextEdit::singleline(&mut self.asset_filter_edit).hint_text("Search assets"),
            );
            if widgets::icon_button(ui, "R", "Refresh assets", true).clicked() {
                self.refresh_assets();
            }
        });

        ui.add_space(theme.spacing.sm);
        widgets::section_label(ui, "IMAGE ASSETS");
        if self.assets.is_empty() {
            widgets::inset_frame().show(ui, |ui| {
                ui.label(egui::RichText::new("No image assets").color(theme.colors.text_muted));
            });
            return;
        }

        let images = self.filtered_assets_by_text();
        widgets::inset_frame().show(ui, |ui| {
            if images.is_empty() {
                ui.label(egui::RichText::new("No image assets").color(theme.colors.text_muted));
                return;
            }

            egui::ScrollArea::vertical()
                .id_salt("library_asset_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for image in images.into_iter().take(32) {
                        let selected =
                            self.selected_asset_path.as_deref() == Some(image.asset_path.as_str());
                        let response = ui.add(
                            egui::Button::selectable(
                                selected,
                                truncate_text(&image.display_name, 24),
                            )
                            .fill(if selected {
                                theme.colors.control_active
                            } else {
                                theme.colors.panel_bg_alt
                            })
                            .stroke(egui::Stroke::new(
                                1.0,
                                if selected {
                                    theme.colors.accent
                                } else {
                                    theme.colors.border
                                },
                            ))
                            .corner_radius(theme.radius.sm)
                            .min_size(egui::vec2(ui.available_width(), 28.0)),
                        );
                        let clicked = response.clicked();
                        response.on_hover_text(image.asset_path.as_str());
                        if clicked {
                            self.choose_asset(image.asset_path, image.display_name.as_str(), true);
                        }
                    }
                });
        });
    }
}
