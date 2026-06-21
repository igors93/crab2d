use std::collections::{BTreeSet, VecDeque};
use std::path::{Path, PathBuf};
use std::time::Instant;

use crab2d_core::{
    animation_system, audio_system::AudioSystem, particle_system::ParticleSystem,
    save_system::GameSave, scene_manager::SceneManager, Engine, EngineConfig, FrameStep,
};
use crab2d_platform::{InputState, KeyCode};
use eframe::egui;

use crate::assets::asset_roots;
use crate::input::collect_runtime_input;
use crate::renderer::EguiRuntimeRenderer;
use crate::scene_defaults::ensure_runtime_defaults;
use crate::scripting::RuntimeScripting;

pub(crate) struct RuntimeApp {
    engine: Engine,
    input: InputState,
    previous_keys: BTreeSet<KeyCode>,
    last_frame: Instant,
    last_step: FrameStep,
    renderer: EguiRuntimeRenderer,
    scripting: RuntimeScripting,
    audio_system: AudioSystem,
    particle_system: ParticleSystem,
    scene_manager: SceneManager,
    #[allow(dead_code)]
    game_save: GameSave,
    debug_overlay: bool,
    fps_history: VecDeque<f32>,
}

impl RuntimeApp {
    pub(crate) fn load(project_path: &Path) -> Result<Self, String> {
        let mut engine = Engine::new(EngineConfig::new("Crab2D Runtime"));
        engine
            .load_project_document(project_path)
            .map_err(|error| error.to_string())?;
        ensure_runtime_defaults(&mut engine);

        let roots = asset_roots(project_path);
        let save_dir = project_path
            .parent()
            .map(|path| path.join("saves"))
            .unwrap_or_else(|| PathBuf::from("saves"));

        Ok(Self {
            engine,
            input: InputState::default(),
            previous_keys: BTreeSet::new(),
            last_frame: Instant::now(),
            last_step: FrameStep::empty(0.0),
            renderer: EguiRuntimeRenderer::new(roots.clone()),
            scripting: RuntimeScripting::new(roots.clone()),
            audio_system: AudioSystem::new(roots.clone()),
            particle_system: ParticleSystem::new(),
            scene_manager: SceneManager::new(roots.clone()),
            game_save: GameSave::new(save_dir),
            debug_overlay: false,
            fps_history: VecDeque::with_capacity(60),
        })
    }

    pub(crate) fn configure_renderer(&mut self, ctx: &egui::Context) {
        self.renderer.configure(ctx);
    }

    fn play_auto_audio(&mut self) {
        let clips: Vec<(String, f32, bool)> = self
            .engine
            .active_scene
            .audios()
            .filter(|(_, audio)| audio.auto_play)
            .map(|(_, audio)| (audio.clip_path.clone(), audio.volume, audio.looping))
            .collect();
        for (path, volume, looping) in clips {
            self.audio_system.play_clip_once(&path, volume, looping);
        }
    }

    fn update_fps(&mut self, delta_seconds: f32) {
        if delta_seconds > 0.0 {
            if self.fps_history.len() >= 60 {
                self.fps_history.pop_front();
            }
            self.fps_history.push_back(1.0 / delta_seconds);
        }
    }

    fn average_fps(&self) -> f32 {
        if self.fps_history.is_empty() {
            0.0
        } else {
            self.fps_history.iter().sum::<f32>() / self.fps_history.len() as f32
        }
    }
}

impl eframe::App for RuntimeApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        let delta_seconds = (now - self.last_frame).as_secs_f32().clamp(0.0, 1.0 / 20.0);
        self.last_frame = now;

        let ctx = ui.ctx().clone();
        collect_runtime_input(&mut self.input, &mut self.previous_keys, &ctx);
        if self.input.was_key_pressed(KeyCode::Escape) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if self.input.was_key_pressed(KeyCode::F1) {
            self.debug_overlay = !self.debug_overlay;
        }

        self.update_fps(delta_seconds);

        match self.engine.tick_with_input(delta_seconds, &self.input) {
            Ok(step) => self.last_step = step,
            Err(error) => eprintln!("Runtime tick failed: {error}"),
        }

        let anim_ended =
            animation_system::tick_animations(&mut self.engine.active_scene, delta_seconds);
        self.last_step.animation_ended = anim_ended;
        self.particle_system
            .tick(&self.engine.active_scene, delta_seconds);
        self.scripting.run_frame(
            &mut self.engine,
            &self.input,
            &self.last_step,
            &mut self.audio_system,
            &mut self.scene_manager,
            delta_seconds,
        );
        self.play_auto_audio();

        if let Some((new_scene, _path)) = self.scene_manager.apply_transition() {
            self.engine.active_scene = new_scene;
            self.scripting.clear();
        }

        let avg_fps = self.average_fps();
        let entity_count = self.engine.active_scene.len();

        self.renderer.draw(
            ui,
            &self.engine.active_scene,
            &self.last_step,
            &self.particle_system,
            self.debug_overlay,
            avg_fps,
            entity_count,
        );
        ctx.request_repaint();
    }
}
