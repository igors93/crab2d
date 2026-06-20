use crab2d_core::{Engine, EngineConfig, ProjectInfo};
use crab2d_platform::{HeadlessShell, PlatformShell};
use crab2d_procgen::{GenerationSettings, StarterVillageGenerator, WorldGenerator};
use crab2d_render::{NullRenderer, Renderer2D};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    Select,
    TileBrush,
    CollisionEdit,
    ProceduralPreview,
}

#[derive(Debug)]
pub struct EditorApp {
    title: String,
    engine: Engine,
    mode: EditorMode,
    renderer: NullRenderer,
    shell: HeadlessShell,
}

impl EditorApp {
    pub fn new(title: impl Into<String>) -> Self {
        let title = title.into();
        Self {
            engine: Engine::new(EngineConfig::new(title.clone())),
            title,
            mode: EditorMode::Select,
            renderer: NullRenderer::default(),
            shell: HeadlessShell,
        }
    }

    pub fn open_empty_project(&mut self, project_name: impl Into<String>) {
        let project = ProjectInfo::new(project_name);
        self.engine.open_project(project);
        self.engine.active_scene.spawn_node("Camera2D");
        self.engine.active_scene.spawn_node("Player");
        self.engine.active_scene.spawn_node("ProceduralWorld");
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

    pub fn run_once(&mut self) {
        let _events = self.shell.poll_events();
        self.engine.tick(1.0 / 60.0);
        self.renderer.begin_frame();
        self.renderer.draw_scene(&self.engine.active_scene);
        let stats = self.renderer.end_frame();

        println!(
            "{} opened '{}' in {:?} mode: {} draw call(s), {} visible node(s)",
            self.title,
            self.engine.project.name,
            self.mode,
            stats.draw_calls,
            stats.sprites
        );
    }
}
