mod app;
mod assets;
mod input;
mod renderer;
mod scene_defaults;
mod scripting;

use std::path::PathBuf;

use app::RuntimeApp;
use crab2d_core::ProjectDocument;
use eframe::egui;

fn main() {
    if let Err(error) = run() {
        eprintln!("Crab2D runtime failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let project_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(ProjectDocument::DEFAULT_FILE_NAME));
    let app = RuntimeApp::load(&project_path)?;

    eframe::run_native(
        "Crab2D Runtime",
        eframe::NativeOptions {
            renderer: eframe::Renderer::Glow,
            viewport: egui::ViewportBuilder::default()
                .with_title("Crab2D Runtime")
                .with_inner_size([960.0, 540.0])
                .with_min_inner_size([640.0, 360.0]),
            ..Default::default()
        },
        Box::new(|cc| {
            let mut app = app;
            app.configure_renderer(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
    .map_err(|error| error.to_string())
}
