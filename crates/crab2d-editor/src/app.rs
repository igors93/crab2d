use std::path::{Path, PathBuf};

use crab2d_core::{Engine, EngineConfig, ProjectDocument, ProjectInfo, ProjectIoError};
use crab2d_platform::{HeadlessShell, PlatformShell};
use crab2d_procgen::{GenerationSettings, StarterVillageGenerator, WorldGenerator as _};
use crab2d_render::{NullRenderer, RenderStats, Renderer2D};
use crab2d_scene::{
    Camera2DComponent, CameraFollowComponent, Collider2DComponent, EntityId, Node2D,
    PlayerControllerComponent, SpriteComponent, TagComponent, TileCell, TilemapComponent,
    Transform2D, TriggerComponent, Velocity2DComponent,
};

use crate::{default_tilemap, EditorProjectSession};
use crate::{
    ensure_project_structure, CommandHistory, CommandHistoryError, EditorCommand,
    EditorCommandError, EditorCommandResult, EditorMode, GameplayPreset, ProjectBootstrap,
    ProjectSessionError, ProjectTemplate,
};

#[derive(Debug)]
pub struct EditorApp {
    title: String,
    engine: Engine,
    session: EditorProjectSession,
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
            session: EditorProjectSession::untitled("Untitled Project"),
            history: CommandHistory::default(),
            title,
            mode: EditorMode::default(),
            renderer: NullRenderer::default(),
            shell: HeadlessShell::default(),
        }
    }

    pub fn open_empty_project(&mut self, project_name: impl Into<String>) {
        let project_name = project_name.into();
        ProjectBootstrap::empty_project(project_name.clone())
            .apply(&mut self.engine)
            .expect("starter project bootstrap should be valid");
        self.session = EditorProjectSession::untitled(project_name);
        self.clear_history();
    }

    pub fn new_project(
        &mut self,
        project_name: impl Into<String>,
        project_root: impl AsRef<Path>,
        template: ProjectTemplate,
    ) -> Result<PathBuf, ProjectSessionError> {
        let project_name = project_name.into();
        let project_root = project_root.as_ref().to_path_buf();
        ensure_project_structure(&project_root)?;
        self.apply_template(project_name.clone(), template)?;
        self.engine.project.root = Some(project_root.clone());
        let project_path = project_root.join(ProjectDocument::DEFAULT_FILE_NAME);
        self.engine.save_project_document(&project_path)?;
        self.session = EditorProjectSession::for_path(project_name, project_path.clone());
        self.clear_history();
        Ok(project_path)
    }

    pub fn save_current_project(&mut self, path: impl AsRef<Path>) -> Result<(), ProjectIoError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(ProjectIoError::from)?;
            }
        }
        self.engine.project.root = path.parent().map(Path::to_path_buf);
        self.engine.save_project_document(path)?;
        self.session.project_name = self.engine.project.name.clone();
        self.session.set_project_file(path);
        Ok(())
    }

    pub fn save_current_project_to_default_file(&mut self) -> Result<(), ProjectIoError> {
        self.save_current_project(ProjectDocument::DEFAULT_FILE_NAME)
    }

    pub fn save_project(&mut self) -> Result<PathBuf, ProjectSessionError> {
        let path = self
            .session
            .project_path
            .clone()
            .ok_or(ProjectSessionError::MissingProjectPath)?;
        self.save_current_project(&path)?;
        Ok(path)
    }

    pub fn save_project_as(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<PathBuf, ProjectSessionError> {
        let path = normalize_project_path(path.as_ref());
        self.save_current_project(&path)?;
        Ok(path)
    }

    pub fn load_project(&mut self, path: impl AsRef<Path>) -> Result<(), ProjectIoError> {
        let path = path.as_ref();
        let document = ProjectDocument::load_from_path(path)?;
        document.apply_to_engine(&mut self.engine);
        self.session = EditorProjectSession::for_path(self.engine.project.name.clone(), path);
        self.clear_history();
        Ok(())
    }

    pub fn execute_command(
        &mut self,
        command: EditorCommand,
    ) -> Result<EditorCommandResult, EditorCommandError> {
        let result = command.apply(&mut self.engine)?;
        self.session.mark_dirty();
        Ok(result)
    }

    pub fn execute_command_with_history(
        &mut self,
        command: EditorCommand,
    ) -> Result<EditorCommandResult, CommandHistoryError> {
        let result = self.history.execute(command, &mut self.engine)?;
        self.session.mark_dirty();
        Ok(result)
    }

    pub fn undo(&mut self) -> Result<(), CommandHistoryError> {
        self.history.undo(&mut self.engine)?;
        self.session.mark_dirty();
        Ok(())
    }

    pub fn redo(&mut self) -> Result<(), CommandHistoryError> {
        self.history.redo(&mut self.engine)?;
        self.session.mark_dirty();
        Ok(())
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
        if before == after {
            return;
        }
        self.history.push_move_node(entity, before, after);
        self.session.mark_dirty();
    }

    pub fn project_name(&self) -> &str {
        &self.engine.project.name
    }

    pub fn project_path(&self) -> Option<&Path> {
        self.session.project_path.as_deref()
    }

    pub fn project_root(&self) -> Option<&Path> {
        self.session.project_root.as_deref()
    }

    pub fn project_session(&self) -> &EditorProjectSession {
        &self.session
    }

    pub fn is_dirty(&self) -> bool {
        self.session.dirty
    }

    pub fn asset_roots(&self) -> Vec<PathBuf> {
        self.session.asset_roots()
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

    pub fn create_preset_node(
        &mut self,
        preset: GameplayPreset,
    ) -> Result<EntityId, CommandHistoryError> {
        let result = self
            .execute_command_with_history(EditorCommand::create_node(preset.default_node_name()))?;
        let EditorCommandResult::CreatedNode(entity) = result else {
            return Err(CommandHistoryError::UnexpectedCommandResult);
        };
        self.execute_command_with_history(EditorCommand::apply_gameplay_preset(entity, preset))?;
        Ok(entity)
    }

    fn apply_template(
        &mut self,
        project_name: String,
        template: ProjectTemplate,
    ) -> Result<(), ProjectSessionError> {
        match template {
            ProjectTemplate::EmptyProject => {
                self.engine.open_project(ProjectInfo::new(project_name));
            }
            ProjectTemplate::TopDownStarter => {
                ProjectBootstrap::empty_project(project_name)
                    .apply(&mut self.engine)
                    .map_err(ProjectSessionError::from)?;
            }
            ProjectTemplate::TilemapStarter => {
                self.engine.open_project(ProjectInfo::new(project_name));
                let camera = self.engine.active_scene.spawn_node("Camera2D");
                self.engine
                    .active_scene
                    .add_camera(camera, Camera2DComponent::default())
                    .map_err(ProjectSessionError::from)?;
                let world = self.engine.active_scene.spawn_node("Tilemap_Ground");
                self.engine
                    .active_scene
                    .add_tag(world, TagComponent::new("world"))
                    .map_err(ProjectSessionError::from)?;
                let mut tilemap = default_tilemap().map_err(ProjectSessionError::from)?;
                for y in 0..tilemap.map_size.height {
                    for x in 0..tilemap.map_size.width {
                        let border = x == 0
                            || y == 0
                            || x == tilemap.map_size.width - 1
                            || y == tilemap.map_size.height - 1;
                        let tile = if border { 3 } else { 0 };
                        tilemap
                            .set_tile("Ground", x, y, Some(TileCell::new(tile)))
                            .map_err(ProjectSessionError::from)?;
                    }
                }
                self.engine
                    .active_scene
                    .add_tilemap(world, tilemap)
                    .map_err(ProjectSessionError::from)?;
            }
        }
        Ok(())
    }

    pub fn preview_procedural_world(&mut self) {
        self.mode = EditorMode::ProceduralPreview;
        let generator = StarterVillageGenerator;
        let _map = generator.generate_scene(&GenerationSettings {
            seed: Some(1),
            scene_name: "GeneratedWorld".to_string(),
            map_width: 64,
            map_height: 64,
            tile_size: 32,
        });
    }

    /// Replace the active scene entirely (e.g. after AI generation).
    /// Clears command history since the previous undo stack is no longer valid.
    pub fn replace_active_scene(&mut self, scene: crab2d_scene::Scene) {
        self.engine.active_scene = scene;
        self.clear_history();
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

fn normalize_project_path(path: &Path) -> PathBuf {
    if path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.ends_with(".crab2d.json"))
        .unwrap_or(false)
    {
        path.to_path_buf()
    } else {
        path.join(ProjectDocument::DEFAULT_FILE_NAME)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::{EditorApp, EditorCommand, GameplayPreset, ProjectSessionError, ProjectTemplate};

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

    #[test]
    fn new_project_creates_structure_and_saves_document() {
        let root = test_project_dir("new-project");
        let mut app = EditorApp::new("Crab2D Editor");

        let project_path = app
            .new_project("My Game", &root, ProjectTemplate::TopDownStarter)
            .expect("new project should save");

        assert_eq!(project_path, root.join("project.crab2d.json"));
        assert!(project_path.exists());
        assert!(root.join("assets/sprites").is_dir());
        assert!(root.join("assets/tilesets").is_dir());
        assert!(root.join("assets/audio").is_dir());
        assert!(root.join("scenes").is_dir());
        assert_eq!(app.project_path(), Some(project_path.as_path()));
        assert!(!app.is_dirty());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn save_without_project_path_requires_save_as() {
        let mut app = EditorApp::new("Crab2D Editor");
        app.open_empty_project("Unsaved");

        let result = app.save_project();

        assert!(matches!(
            result,
            Err(ProjectSessionError::MissingProjectPath)
        ));
    }

    #[test]
    fn editing_marks_dirty_and_save_marks_clean() {
        let path = test_project_path("dirty-save");
        let mut app = EditorApp::new("Crab2D Editor");
        app.open_empty_project("Dirty Project");
        let player = app
            .scene_nodes()
            .iter()
            .find(|node| node.name == "Player")
            .expect("player exists")
            .id;

        app.execute_command_with_history(EditorCommand::rename_node(player, "Hero"))
            .expect("rename should execute");
        assert!(app.is_dirty());

        app.save_project_as(&path).expect("save as should succeed");
        assert!(!app.is_dirty());

        let mut loaded = EditorApp::new("Crab2D Editor");
        loaded.load_project(&path).expect("project should load");
        assert!(loaded.scene_nodes().iter().any(|node| node.name == "Hero"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn top_down_template_does_not_reference_missing_tileset() {
        let mut app = EditorApp::new("Crab2D Editor");
        app.open_empty_project("Starter");
        let tilemap = app
            .first_tilemap_node()
            .and_then(|entity| app.node_tilemap(entity))
            .expect("starter tilemap exists");

        assert!(tilemap.tileset.is_none());
    }

    #[test]
    fn top_down_player_preset_attaches_runtime_components() {
        let mut app = EditorApp::new("Crab2D Editor");
        app.open_empty_project("Preset Project");

        let entity = app
            .create_preset_node(GameplayPreset::TopDownPlayer)
            .expect("preset should create");

        assert!(app.node_sprite(entity).is_some());
        assert!(app.node_collider(entity).is_some());
        assert!(app.node_velocity(entity).is_some());
        assert!(app.node_player_controller(entity).is_some());
        assert_eq!(app.node_tag(entity).expect("tag").tag, "player");
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

    fn test_project_dir(label: &str) -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after Unix epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("crab2d-{label}-{}-{now}", std::process::id()))
    }
}
