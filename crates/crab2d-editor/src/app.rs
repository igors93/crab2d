use crab2d_core::{Engine, EngineConfig};
use crab2d_platform::{HeadlessShell, PlatformShell};
use crab2d_procgen::{GenerationSettings, StarterVillageGenerator, WorldGenerator};
use crab2d_render::{NullRenderer, RenderStats, Renderer2D};

use crate::{EditorMode, ProjectBootstrap};

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
}
