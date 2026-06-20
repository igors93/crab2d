fn main() {
    let mut app = crab2d_editor::EditorApp::new("Crab2D Editor");
    app.open_empty_project("Untitled Crab2D Project");
    app.run_once();
}
