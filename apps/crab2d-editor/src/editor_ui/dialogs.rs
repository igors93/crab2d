use super::*;

impl Crab2DEditorUi {
    pub(super) fn show_project_dialogs(&mut self, ctx: &egui::Context) {
        let Some(dialog) = self.active_dialog else {
            return;
        };

        match dialog {
            ProjectDialog::NewProject => {
                egui::Window::new("New Project")
                    .collapsible(false)
                    .resizable(false)
                    .default_width(430.0)
                    .show(ctx, |ui| {
                        widgets::property_row(ui, "Name", |ui| {
                            let response = ui.add_sized(
                                [ui.available_width().max(120.0), 24.0],
                                egui::TextEdit::singleline(&mut self.project_name_edit),
                            );
                            if response.changed() && !self.project_name_edit.trim().is_empty() {
                                self.project_path_edit =
                                    default_project_path(self.project_name_edit.trim());
                            }
                        });
                        widgets::property_row(ui, "Folder", |ui| {
                            ui.add_sized(
                                [ui.available_width().max(120.0), 24.0],
                                egui::TextEdit::singleline(&mut self.project_path_edit),
                            );
                        });
                        widgets::property_row(ui, "Template", |ui| {
                            ui.vertical(|ui| {
                                for template in ProjectTemplate::ALL {
                                    ui.radio_value(
                                        &mut self.selected_template,
                                        template,
                                        template.label(),
                                    );
                                }
                            });
                        });
                        ui.separator();
                        ui.horizontal(|ui| {
                            if widgets::toolbar_button(ui, "Create", "Create project", true, false)
                                .clicked()
                            {
                                self.create_new_project_from_dialog();
                            }
                            if widgets::toolbar_button(ui, "Cancel", "Close dialog", true, false)
                                .clicked()
                            {
                                self.active_dialog = None;
                            }
                        });
                    });
            }
            ProjectDialog::OpenProject => {
                egui::Window::new("Open Project")
                    .collapsible(false)
                    .resizable(false)
                    .default_width(460.0)
                    .show(ctx, |ui| {
                        widgets::property_row(ui, "Project Path", |ui| {
                            ui.add_sized(
                                [ui.available_width().max(160.0), 24.0],
                                egui::TextEdit::singleline(&mut self.open_path_edit),
                            );
                        });
                        ui.separator();
                        ui.horizontal(|ui| {
                            if widgets::toolbar_button(ui, "Open", "Open project", true, false)
                                .clicked()
                            {
                                self.open_project_from_dialog();
                            }
                            if widgets::toolbar_button(ui, "Cancel", "Close dialog", true, false)
                                .clicked()
                            {
                                self.active_dialog = None;
                            }
                        });
                    });
            }
            ProjectDialog::SaveAs => {
                egui::Window::new("Save As")
                    .collapsible(false)
                    .resizable(false)
                    .default_width(460.0)
                    .show(ctx, |ui| {
                        widgets::property_row(ui, "Project Path", |ui| {
                            ui.add_sized(
                                [ui.available_width().max(160.0), 24.0],
                                egui::TextEdit::singleline(&mut self.save_as_path_edit),
                            );
                        });
                        ui.separator();
                        ui.horizontal(|ui| {
                            if widgets::toolbar_button(ui, "Save", "Save project", true, false)
                                .clicked()
                            {
                                self.save_as_from_dialog();
                            }
                            if widgets::toolbar_button(ui, "Cancel", "Close dialog", true, false)
                                .clicked()
                            {
                                self.active_dialog = None;
                            }
                        });
                    });
            }
            ProjectDialog::SaveBeforePlay => {
                egui::Window::new("Save Before Running")
                    .collapsible(false)
                    .resizable(false)
                    .default_width(360.0)
                    .show(ctx, |ui| {
                        ui.label("The current project has unsaved changes.");
                        ui.separator();
                        ui.horizontal(|ui| {
                            if widgets::toolbar_button(
                                ui,
                                "Save & Play",
                                "Save and run",
                                true,
                                false,
                            )
                            .clicked()
                            {
                                self.save_and_play();
                            }
                            if widgets::toolbar_button(
                                ui,
                                "Run Saved",
                                "Run the last saved version",
                                self.app.project_path().is_some(),
                                false,
                            )
                            .clicked()
                            {
                                self.active_dialog = None;
                                self.launch_runtime();
                            }
                            if widgets::toolbar_button(ui, "Cancel", "Close dialog", true, false)
                                .clicked()
                            {
                                self.active_dialog = None;
                            }
                        });
                    });
            }
        }
    }
}
