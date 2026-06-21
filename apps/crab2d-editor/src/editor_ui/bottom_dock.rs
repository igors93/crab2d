use super::*;

impl Crab2DEditorUi {
    pub(super) fn show_asset_dock_contents(&mut self, ui: &mut egui::Ui) {
        let theme = theme();
        egui::Frame::new()
            .fill(theme.colors.panel_header_bg)
            .stroke(egui::Stroke::new(1.0, theme.colors.border))
            .inner_margin(egui::Margin::symmetric(6, 4))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = theme.spacing.xs;
                    self.show_bottom_tab_button(ui, BottomDockTab::TilePalette, "Tile Palette");
                    self.show_bottom_tab_button(ui, BottomDockTab::Assets, "Assets");
                    self.show_bottom_tab_button(ui, BottomDockTab::Output, "Output");
                    ui.add_enabled(
                        false,
                        egui::Button::selectable(false, "Debugger")
                            .corner_radius(theme.radius.sm)
                            .min_size(egui::vec2(80.0, 26.0))
                            .fill(theme.colors.panel_bg_alt)
                            .stroke(egui::Stroke::new(1.0, theme.colors.border)),
                    )
                    .on_hover_text("Debugger (coming soon)");

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if widgets::toolbar_button(
                            ui,
                            if self.asset_dock_collapsed {
                                "Expand"
                            } else {
                                "Collapse"
                            },
                            "Collapse or expand dock",
                            true,
                            false,
                        )
                        .clicked()
                        {
                            self.asset_dock_collapsed = !self.asset_dock_collapsed;
                        }
                        widgets::chip(
                            ui,
                            format!("{} assets", self.assets.images().len()).as_str(),
                            StatusTone::Info,
                        );
                        widgets::status_badge(ui, &self.status, self.status_tone);
                    });
                });
            });
        if self.asset_dock_collapsed {
            return;
        }

        ui.add_space(theme.spacing.sm);

        match self.bottom_tab {
            BottomDockTab::TilePalette => self.show_tile_palette(ui),
            BottomDockTab::Assets => self.show_asset_browser(ui),
            BottomDockTab::Output => self.show_output_panel(ui),
        }
    }

    pub(super) fn show_bottom_tab_button(
        &mut self,
        ui: &mut egui::Ui,
        tab: BottomDockTab,
        label: &str,
    ) {
        if widgets::segment_button(ui, label, self.bottom_tab == tab).clicked() {
            self.bottom_tab = tab;
        }
    }

    pub(super) fn show_tile_palette(&mut self, ui: &mut egui::Ui) {
        let theme = theme();
        widgets::inset_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                widgets::section_label(ui, "GROUND");
                widgets::chip(ui, self.active_tool.label(), StatusTone::Info);
                widgets::chip(
                    ui,
                    format!("Tile {}", self.selected_tile_index).as_str(),
                    StatusTone::Info,
                );
            });
            ui.add_space(theme.spacing.sm);
            ui.horizontal_wrapped(|ui| {
                for index in 0..16 {
                    let selected = self.selected_tile_index == index;
                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(theme.sizing.tile_button_size, theme.sizing.tile_button_size),
                        egui::Sense::click(),
                    );
                    let painter = ui.painter();
                    painter.rect_filled(rect, theme.radius.sm, tile_color(index));
                    painter.rect_stroke(
                        rect,
                        theme.radius.sm,
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
                    painter.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        index.to_string(),
                        egui::FontId::monospace(11.0),
                        egui::Color32::from_rgba_unmultiplied(8, 12, 14, 180),
                    );
                    if response.clicked() {
                        self.selected_tile_index = index;
                        self.active_tool = EditorTool::TileBrush;
                        self.set_status(format!("Tile {index} selected"));
                    }
                }
            });
        });
    }

    pub(super) fn show_asset_browser(&mut self, ui: &mut egui::Ui) {
        let theme = theme();
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(118.0);
                for category in AssetCategory::ALL {
                    if widgets::segment_button(
                        ui,
                        category.label(),
                        self.asset_category == category,
                    )
                    .clicked()
                    {
                        self.asset_category = category;
                    }
                }
            });
            ui.separator();
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = theme.spacing.xs;
                    widgets::toolbar_button(
                        ui,
                        "Import",
                        "Import asset into project",
                        false,
                        false,
                    )
                    .clicked();
                    if widgets::icon_button(ui, "↺", "Refresh assets", true).clicked() {
                        self.refresh_assets();
                    }
                    ui.add_sized(
                        [220.0, 26.0],
                        egui::TextEdit::singleline(&mut self.asset_filter_edit)
                            .hint_text("Search assets..."),
                    );
                    ui.separator();
                    if widgets::segment_button(
                        ui,
                        "Images",
                        self.asset_tab == AssetBrowserTab::Images,
                    )
                    .clicked()
                    {
                        self.asset_tab = AssetBrowserTab::Images;
                    }
                    if widgets::segment_button(
                        ui,
                        "Broken",
                        self.asset_tab == AssetBrowserTab::Broken,
                    )
                    .clicked()
                    {
                        self.asset_tab = AssetBrowserTab::Broken;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if widgets::toolbar_button(
                            ui,
                            "Clear",
                            "Clear asset selection",
                            true,
                            false,
                        )
                        .clicked()
                        {
                            self.selected_asset_path = None;
                            self.asset_filter_edit.clear();
                            self.set_status("Asset selection cleared");
                        }
                    });
                });
                ui.add_space(theme.spacing.sm);

                let images = self.filtered_assets_for_current_tab(ui.ctx());
                if images.is_empty() {
                    widgets::inset_frame().show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("No matching image assets")
                                .color(theme.colors.text_muted),
                        );
                    });
                    return;
                }

                egui::ScrollArea::horizontal()
                    .id_salt("asset_browser_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for image in images {
                                let asset_path = image.asset_path.clone();
                                let display_name = image.display_name.clone();
                                let result = self.image_asset_card(ui, &image);
                                if result.clicked {
                                    if let Some(error) = result.load_error {
                                        self.selected_asset_path = Some(asset_path);
                                        self.report_asset_error(error);
                                    } else {
                                        self.choose_asset(asset_path, display_name.as_str(), true);
                                    }
                                }
                            }
                        });
                    });
            });
        });
    }

    pub(super) fn show_output_panel(&mut self, ui: &mut egui::Ui) {
        widgets::inset_frame().show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("output_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for line in self.output.iter().rev() {
                        ui.colored_label(output_color(line), line);
                    }
                });
        });
    }
    pub(super) fn image_asset_card(
        &mut self,
        ui: &mut egui::Ui,
        image: &ImageAsset,
    ) -> AssetCardResult {
        let theme = theme();
        let selected = self.selected_asset_path.as_deref() == Some(image.asset_path.as_str());
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(
                theme.sizing.asset_card_width,
                theme.sizing.asset_card_height,
            ),
            egui::Sense::click(),
        );
        let hovered = response.hovered();
        let clicked = response.clicked();
        let painter = ui.painter_at(rect);
        let fill = if selected {
            theme.colors.control_active
        } else if hovered {
            theme.colors.control_hover
        } else {
            theme.colors.panel_bg_alt
        };

        painter.rect_filled(rect, theme.radius.md, fill);
        painter.rect_stroke(
            rect,
            theme.radius.md,
            egui::Stroke::new(
                if selected { 2.0 } else { 1.0 },
                if selected {
                    theme.colors.accent
                } else {
                    theme.colors.border
                },
            ),
            egui::StrokeKind::Inside,
        );

        let thumbnail_rect = egui::Rect::from_min_size(
            rect.left_top() + egui::vec2(8.0, 8.0),
            egui::vec2(rect.width() - 16.0, 76.0),
        );
        painter.rect_filled(thumbnail_rect, theme.radius.sm, theme.colors.app_bg);

        let mut load_error = None;
        match self.textures.load(ui.ctx(), &image.asset_path) {
            TextureLookup::Loaded(texture) => {
                let image_rect = fit_rect(texture.size_vec2(), thumbnail_rect.shrink(6.0));
                painter.image(
                    texture.id(),
                    image_rect,
                    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }
            TextureLookup::Failed(error) => {
                draw_missing_texture_marker(&painter, thumbnail_rect.shrink(12.0), "!");
                load_error = Some(format!(
                    "Asset failed to load '{}': {error}",
                    image.asset_path
                ));
            }
            TextureLookup::Missing => {
                draw_missing_texture_marker(&painter, thumbnail_rect.shrink(12.0), "?");
            }
        }

        let name = truncate_text(&image.display_name, 18);
        painter.text(
            rect.left_top() + egui::vec2(10.0, 94.0),
            egui::Align2::LEFT_TOP,
            name,
            egui::FontId::proportional(12.0),
            theme.colors.text,
        );
        painter.text(
            rect.left_bottom() + egui::vec2(10.0, -22.0),
            egui::Align2::LEFT_TOP,
            truncate_text(&image.asset_path, 22),
            egui::FontId::monospace(10.0),
            theme.colors.text_muted,
        );

        let tooltip = if let Some(error) = load_error.as_deref() {
            format!("{}\n{error}", image.asset_path)
        } else {
            image.asset_path.clone()
        };
        response.on_hover_text(tooltip);

        AssetCardResult {
            clicked,
            load_error,
        }
    }

    pub(super) fn apply_sprite(&mut self, entity: EntityId, sprite_path: String) {
        match self
            .app
            .execute_command_with_history(EditorCommand::attach_sprite(entity, sprite_path))
        {
            Ok(_) => {
                self.sync_selected_buffers();
                self.set_success("Sprite applied");
            }
            Err(error) => self.set_error(format!("{error}")),
        }
    }

    pub(super) fn remove_component(
        &mut self,
        entity: EntityId,
        component: EditorComponentKind,
        success_message: &str,
    ) {
        match self
            .app
            .execute_command_with_history(EditorCommand::remove_component(entity, component))
        {
            Ok(_) => {
                self.sync_selected_buffers();
                self.set_success(success_message);
            }
            Err(error) => self.set_error(format!("{error}")),
        }
    }

    pub(super) fn choose_asset(
        &mut self,
        asset_path: String,
        display_name: &str,
        apply_to_selected: bool,
    ) {
        self.selected_asset_path = Some(asset_path.clone());
        if apply_to_selected {
            if let Some(entity) = self.selected {
                self.sprite_edit = asset_path.clone();
                self.apply_sprite(entity, asset_path);
                return;
            }
        }
        self.set_status(format!("Asset selected: {display_name}"));
    }

    pub(super) fn refresh_assets(&mut self) {
        let roots = asset_roots(self.app.asset_roots());
        self.textures.set_asset_roots(roots.clone());
        self.assets = ImageAssetCatalog::scan(&roots);
        self.set_success("Assets refreshed");
    }

    pub(super) fn filtered_assets_by_text(&self) -> Vec<ImageAsset> {
        let filter = self.asset_filter_edit.trim().to_lowercase();
        self.assets
            .images()
            .iter()
            .filter(|image| {
                self.asset_category_matches(image)
                    && (filter.is_empty()
                        || image.display_name.to_lowercase().contains(filter.as_str())
                        || image.asset_path.to_lowercase().contains(filter.as_str()))
            })
            .cloned()
            .collect()
    }

    fn asset_category_matches(&self, image: &ImageAsset) -> bool {
        let path = image.asset_path.to_lowercase();
        match self.asset_category {
            AssetCategory::All => true,
            AssetCategory::Sprites => {
                path.contains("sprite") || path.contains("player") || path.contains("character")
            }
            AssetCategory::Tilemaps => {
                path.contains("tile") || path.contains("tileset") || path.contains("map")
            }
            AssetCategory::Ui => {
                path.contains("ui") || path.contains("icon") || path.contains("hud")
            }
            AssetCategory::Other => {
                !(path.contains("sprite")
                    || path.contains("player")
                    || path.contains("character")
                    || path.contains("tile")
                    || path.contains("tileset")
                    || path.contains("map")
                    || path.contains("ui")
                    || path.contains("icon")
                    || path.contains("hud"))
            }
        }
    }

    pub(super) fn filtered_assets_for_current_tab(
        &mut self,
        ctx: &egui::Context,
    ) -> Vec<ImageAsset> {
        let images = self.filtered_assets_by_text();
        if self.asset_tab != AssetBrowserTab::Broken {
            return images;
        }

        let images = images
            .into_iter()
            .filter(|image| {
                matches!(
                    self.textures.load(ctx, &image.asset_path),
                    TextureLookup::Failed(_)
                )
            })
            .collect();
        images
    }
}
