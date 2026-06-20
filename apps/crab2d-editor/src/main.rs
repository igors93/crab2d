fn main() {
    let mut app = crab2d_editor::EditorApp::new("Crab2D Editor");
    app.open_empty_project("Untitled Crab2D Project");

    if std::env::args().any(|arg| arg == "--save-starter-project") {
        app.save_current_project_to_default_file()
            .expect("starter project should save");
        println!("Saved starter project to project.crab2d.json");
        return;
    }

    app.run_once();
}
