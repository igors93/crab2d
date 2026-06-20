use std::path::Path;

use crab2d_core::{Engine, EngineConfig, ProjectDocument, ProjectIoError};
use crab2d_platform::{HeadlessShell, PlatformShell};
use crab2d_procgen::{GenerationSettings, StarterVillageGenerator, WorldGenerator};
use crab2d_render::{NullRenderer, RenderStats, Renderer2D};

use crate::{
    CommandHistory, CommandHistoryError, EditorCommand, EditorCommandError, EditorCommandResult,
    EditorMode, ProjectBootstrap,
};

#[derive(Debug)]
pub struct EditorApp {
    title: String,
    engine: Engine,
    history: CommandHistory,
    mode: EditorMode,
    renderer: NullRenderer,
    shell: HeadlessShell,
}

impl EditorApp {
    pub fn new(title: impl Into<String>) -> Self {
        let title = title.into();
        Self {
            engine: Engine::new(EngineConfig::new(title.clone())),
            history: CommandHistory::default(),
            title,
            mode: EditorMode::default(),
            renderer: NullRenderer::default(),
            shell: HeadlessShell::default(),
        }
    }

    pub fn open_empty_project(&mut self, project_name: impl Into<String>) {
        ProjectBootstrap::empty_project(project_name)
            .apply(&mut self.engine)
            .expect("starter project bootstrap should be valid");
    }

    pub fn save_current_project(&self, path: impl AsRef<Path>) -> Result<(), ProjectIoError> {
        self.engine.save_project_document(path)
    }

    pub fn save_current_project_to_default_file(&self) -> Result<(), ProjectIoError> {
        self.save_current_project(ProjectDocument::DEFAULT_FILE_NAME)
    }

    pub fn load_project(&mut self, path: impl AsRef<Path>) -> Result<(), ProjectIoError> {
        self.engine.load_project_document(path)
    }

    pub fn execute_command(
        &mut self,
        command: EditorCommand,
    ) -> Result<EditorCommandResult, EditorCommandError> {
        command.apply(&mut self.engine)
    }

    pub fn execute_command_with_history(
        &mut self,
        command: EditorCommand,
    ) -> Result<EditorCommandResult, CommandHistoryError> {
        self.history.execute(command, &mut self.engine)
    }

    pub fn undo(&mut self) -> Result<(), CommandHistoryError> {
        self.history.undo(&mut self.engine)
    }

    pub fn redo(&mut self) -> Result<(), CommandHistoryError> {
        self.history.redo(&mut self.engine)
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    pub fn preview_procedural_world(&mut self) {
        self.mode = EditorMode::ProceduralPreview;
        let generator = StarterVillageGenerator;
        let _map = generator.generate(GenerationSettings {
            seed: 1,
            width: 64,
            height: 64,
        });
    }

    pub fn render_frame(&mut self) -> RenderStats {
        self.renderer.begin_frame();
        self.renderer.draw_scene(&self.engine.active_scene);
        self.renderer.end_frame()
    }

    pub fn run_once(&mut self) {
        let _events = self.shell.poll_events();
        self.engine.tick(1.0 / 60.0);
        let stats = self.render_frame();

        println!(
            "{} opened '{}' in {:?} mode: {} draw call(s), {} visible sprite(s)",
            self.title, self.engine.project.name, self.mode, stats.draw_calls, stats.sprites
        );
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::EditorApp;

    #[test]
    fn editor_can_open_empty_project() {
        let mut app = EditorApp::new("Crab2D Editor");

        app.open_empty_project("Test Project");

        assert_eq!(app.engine.project.name, "Test Project");
        assert_eq!(app.engine.active_scene.len(), 3);
        assert_eq!(app.engine.active_scene.sprites().count(), 1);
    }

    #[test]
    fn starter_scene_produces_one_visible_sprite_and_one_draw_call() {
        let mut app = EditorApp::new("Crab2D Editor");
        app.open_empty_project("Test Project");

        let stats = app.render_frame();

        assert_eq!(stats.sprites, 1);
        assert_eq!(stats.draw_calls, 1);
    }

    #[test]
    fn editor_can_save_and_load_current_project() {
        let path = test_project_path("editor-save-load");
        let mut app = EditorApp::new("Crab2D Editor");
        app.open_empty_project("Saved From Editor");

        app.save_current_project(&path)
            .expect("project should save from editor");

        let mut loaded = EditorApp::new("Crab2D Editor");
        loaded
            .load_project(&path)
            .expect("project should load into editor");

        assert_eq!(loaded.engine.project.name, "Saved From Editor");
        assert_eq!(loaded.engine.active_scene.len(), 3);
        assert_eq!(loaded.engine.active_scene.sprites().count(), 1);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn editor_app_executes_editor_commands() {
        let mut app = EditorApp::new("Crab2D Editor");

        let result = app
            .execute_command(crate::EditorCommand::create_node("Enemy"))
            .expect("command should succeed");

        let crate::EditorCommandResult::CreatedNode(enemy) = result else {
            panic!("create node should return the created entity id");
        };
        assert_eq!(
            app.engine
                .active_scene
                .node(enemy)
                .expect("node exists")
                .name,
            "Enemy"
        );
    }

    #[test]
    fn editor_app_undoes_and_redoes_history_commands() {
        let mut app = EditorApp::new("Crab2D Editor");

        let result = app
            .execute_command_with_history(crate::EditorCommand::create_node("Enemy"))
            .expect("command should succeed");
        let crate::EditorCommandResult::CreatedNode(enemy) = result else {
            panic!("create node should return the created entity id");
        };

        assert!(app.can_undo());
        assert!(!app.can_redo());

        app.undo().expect("undo should succeed");

        assert!(app.engine.active_scene.node(enemy).is_none());
        assert!(!app.can_undo());
        assert!(app.can_redo());

        app.redo().expect("redo should succeed");

        assert!(app.engine.active_scene.find_node_by_name("Enemy").is_some());
    }

    fn test_project_path(label: &str) -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after Unix epoch")
            .as_nanos();

        std::env::temp_dir().join(format!(
            "crab2d-{label}-{}-{now}.crab2d.json",
            std::process::id()
        ))
    }
}
