mod editor_ui;

use editor_ui::Crab2DEditorUi;

fn main() -> eframe::Result<()> {
    if std::env::args().any(|arg| arg == "--save-starter-project") {
        save_starter_project_headless();
        return Ok(());
    }

    eframe::run_native(
        "Crab2D Editor",
        eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_title("Crab2D Editor")
                .with_inner_size([1440.0, 840.0])
                .with_min_inner_size([960.0, 600.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(Crab2DEditorUi::new(cc)))),
    )
}

fn save_starter_project_headless() {
    let mut app = crab2d_editor::EditorApp::new("Crab2D Editor");
    app.open_empty_project("Untitled Crab2D Project");
    app.save_current_project_to_default_file()
        .expect("starter project should save");
    println!("Saved starter project to project.crab2d.json");
}
