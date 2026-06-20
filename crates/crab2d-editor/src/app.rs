use std::path::Path;

use crab2d_core::{Engine, EngineConfig, ProjectDocument, ProjectIoError};
use crab2d_platform::{HeadlessShell, PlatformShell};
use crab2d_procgen::{GenerationSettings, StarterVillageGenerator, WorldGenerator};
use crab2d_render::{NullRenderer, RenderStats, Renderer2D};
use crab2d_scene::{
    Camera2DComponent, CameraFollowComponent, Collider2DComponent, EntityId, Node2D,
    PlayerControllerComponent, SpriteComponent, TagComponent, TilemapComponent, Transform2D,
    TriggerComponent, Velocity2DComponent,
};

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
        self.clear_history();
    }

    pub fn save_current_project(&self, path: impl AsRef<Path>) -> Result<(), ProjectIoError> {
        self.engine.save_project_document(path)
    }

    pub fn save_current_project_to_default_file(&self) -> Result<(), ProjectIoError> {
        self.save_current_project(ProjectDocument::DEFAULT_FILE_NAME)
    }

    pub fn load_project(&mut self, path: impl AsRef<Path>) -> Result<(), ProjectIoError> {
        self.engine.load_project_document(path)?;
        self.clear_history();
        Ok(())
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

    pub fn clear_history(&mut self) {
        self.history = CommandHistory::default();
    }

    pub fn record_move_node(&mut self, entity: EntityId, before: Transform2D, after: Transform2D) {
        self.history.push_move_node(entity, before, after);
    }

    pub fn project_name(&self) -> &str {
        &self.engine.project.name
    }

    pub fn scene_nodes(&self) -> &[Node2D] {
        self.engine.active_scene.nodes()
    }

    pub fn find_node(&self, id: EntityId) -> Option<&Node2D> {
        self.engine.active_scene.node(id)
    }

    pub fn node_transform(&self, id: EntityId) -> Option<Transform2D> {
        self.engine.active_scene.node(id).map(|node| node.transform)
    }

    pub fn node_tag(&self, id: EntityId) -> Option<&TagComponent> {
        self.engine.active_scene.tag(id)
    }

    pub fn node_sprite(&self, id: EntityId) -> Option<&SpriteComponent> {
        self.engine.active_scene.sprite(id)
    }

    pub fn node_camera(&self, id: EntityId) -> Option<&Camera2DComponent> {
        self.engine.active_scene.camera(id)
    }

    pub fn node_tilemap(&self, id: EntityId) -> Option<&TilemapComponent> {
        self.engine.active_scene.tilemap(id)
    }

    pub fn node_velocity(&self, id: EntityId) -> Option<&Velocity2DComponent> {
        self.engine.active_scene.velocity(id)
    }

    pub fn node_collider(&self, id: EntityId) -> Option<&Collider2DComponent> {
        self.engine.active_scene.collider(id)
    }

    pub fn node_player_controller(&self, id: EntityId) -> Option<&PlayerControllerComponent> {
        self.engine.active_scene.player_controller(id)
    }

    pub fn node_camera_follow(&self, id: EntityId) -> Option<&CameraFollowComponent> {
        self.engine.active_scene.camera_follow(id)
    }

    pub fn node_trigger(&self, id: EntityId) -> Option<&TriggerComponent> {
        self.engine.active_scene.trigger(id)
    }

    pub fn first_tilemap_node(&self) -> Option<EntityId> {
        self.engine
            .active_scene
            .tilemaps()
            .next()
            .map(|(entity, _)| entity)
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
        self.engine
            .tick(1.0 / 60.0)
            .expect("fixed editor tick should be valid");
        let stats = self.render_frame();

        println!(
            "{} opened '{}' in {:?} mode: {} draw call(s), {} visible sprite(s), {} tilemap(s)",
            self.title,
            self.engine.project.name,
            self.mode,
            stats.draw_calls,
            stats.sprites,
            stats.tilemaps
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
    fn editor_can_open_empty_project_with_tilemap() {
        let mut app = EditorApp::new("Crab2D Editor");

        app.open_empty_project("Test Project");

        assert_eq!(app.project_name(), "Test Project");
        assert_eq!(app.scene_nodes().len(), 4);
        assert_eq!(
            app.scene_nodes()
                .iter()
                .filter(|node| app.node_tilemap(node.id).is_some())
                .count(),
            1
        );
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

        assert_eq!(loaded.project_name(), "Saved From Editor");
        assert_eq!(loaded.scene_nodes().len(), 4);
        assert!(loaded.first_tilemap_node().is_some());

        let _ = fs::remove_file(path);
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
