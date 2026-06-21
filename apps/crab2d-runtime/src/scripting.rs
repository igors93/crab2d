use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::SystemTime;

use crab2d_core::{
    audio_system::AudioSystem,
    scene_manager::SceneManager,
    script_runtime::{ScriptContext, ScriptOutput, ScriptRuntime},
    CollisionAxis, Engine, FrameStep, SolidObstacle,
};
use crab2d_platform::{InputState, MouseButton};
use crab2d_scene::EntityId;

use crate::assets::resolve_path;
use crate::input::key_name;

pub(crate) struct RuntimeScripting {
    runtime: ScriptRuntime,
    scripts_started: BTreeSet<(u64, String)>,
    script_mtimes: BTreeMap<String, SystemTime>,
    asset_roots: Vec<PathBuf>,
}

impl RuntimeScripting {
    pub(crate) fn new(asset_roots: Vec<PathBuf>) -> Self {
        Self {
            runtime: ScriptRuntime::new(),
            scripts_started: BTreeSet::new(),
            script_mtimes: BTreeMap::new(),
            asset_roots,
        }
    }

    pub(crate) fn clear(&mut self) {
        self.scripts_started.clear();
        self.runtime.unload_all();
    }

    pub(crate) fn run_frame(
        &mut self,
        engine: &mut Engine,
        input: &InputState,
        last_step: &FrameStep,
        audio_system: &mut AudioSystem,
        scene_manager: &mut SceneManager,
        delta: f32,
    ) {
        let behavior_entities: Vec<(EntityId, String, bool)> = engine
            .active_scene
            .behaviors()
            .map(|(entity, behavior)| (entity, behavior.script_path.clone(), behavior.enabled))
            .collect();

        self.reload_changed_scripts(&behavior_entities);
        self.fire_start_events(
            engine,
            input,
            audio_system,
            scene_manager,
            &behavior_entities,
        );
        self.fire_update_events(
            engine,
            input,
            audio_system,
            scene_manager,
            &behavior_entities,
            delta,
        );
        self.fire_trigger_events(engine, input, audio_system, scene_manager, last_step);
        self.fire_animation_events(engine, input, audio_system, scene_manager, last_step);
        self.fire_collision_events(engine, input, audio_system, scene_manager, last_step);
    }

    fn reload_changed_scripts(&mut self, behavior_entities: &[(EntityId, String, bool)]) {
        for (_, path, _) in behavior_entities {
            let full = resolve_path(&self.asset_roots, path);
            let mtime = std::fs::metadata(&full)
                .and_then(|metadata| metadata.modified())
                .ok();
            let needs_load = if let Some(mtime) = mtime {
                let changed = self
                    .script_mtimes
                    .get(path.as_str())
                    .map(|&previous| previous != mtime)
                    .unwrap_or(true);
                if changed {
                    self.script_mtimes.insert(path.clone(), mtime);
                }
                changed || !self.runtime.is_loaded(path)
            } else {
                !self.runtime.is_loaded(path)
            };

            if needs_load {
                match std::fs::read_to_string(&full) {
                    Ok(source) => {
                        if let Err(error) = self.runtime.load_script(path, &source) {
                            eprintln!("Script '{path}' failed to load: {error}");
                        } else {
                            self.scripts_started
                                .retain(|(_, started_path)| started_path != path);
                        }
                    }
                    Err(error) => {
                        eprintln!("Script '{path}' not found at {}: {error}", full.display())
                    }
                }
            }
        }
    }

    fn fire_start_events(
        &mut self,
        engine: &mut Engine,
        input: &InputState,
        audio_system: &mut AudioSystem,
        scene_manager: &mut SceneManager,
        behavior_entities: &[(EntityId, String, bool)],
    ) {
        for (entity, path, enabled) in behavior_entities {
            if !enabled {
                continue;
            }
            let key = (entity.raw(), path.clone());
            if self.scripts_started.contains(&key) {
                continue;
            }

            let ctx = script_context(engine, *entity, input);
            match self.runtime.call_on_start(path, &ctx) {
                Ok(output) => {
                    apply_script_output(engine, audio_system, scene_manager, *entity, output)
                }
                Err(error) => eprintln!("[Script ERROR] {error}"),
            }
            self.scripts_started.insert(key);
        }
    }

    fn fire_update_events(
        &mut self,
        engine: &mut Engine,
        input: &InputState,
        audio_system: &mut AudioSystem,
        scene_manager: &mut SceneManager,
        behavior_entities: &[(EntityId, String, bool)],
        delta: f32,
    ) {
        for (entity, path, enabled) in behavior_entities {
            if !enabled {
                continue;
            }
            let ctx = script_context(engine, *entity, input);
            match self.runtime.call_on_update(path, &ctx, delta) {
                Ok(output) => {
                    apply_script_output(engine, audio_system, scene_manager, *entity, output)
                }
                Err(error) => eprintln!("[Script ERROR] {error}"),
            }
        }
    }

    fn fire_trigger_events(
        &mut self,
        engine: &mut Engine,
        input: &InputState,
        audio_system: &mut AudioSystem,
        scene_manager: &mut SceneManager,
        last_step: &FrameStep,
    ) {
        for trigger in last_step.triggers.clone() {
            for entity in [trigger.trigger_entity, trigger.activator] {
                let Some(behavior) = engine.active_scene.behavior(entity).cloned() else {
                    continue;
                };
                if !behavior.enabled {
                    continue;
                }
                let ctx = script_context(engine, entity, input);
                match self
                    .runtime
                    .call_on_trigger(&behavior.script_path, &ctx, &trigger.name)
                {
                    Ok(output) => {
                        apply_script_output(engine, audio_system, scene_manager, entity, output)
                    }
                    Err(error) => eprintln!("[Script ERROR] {error}"),
                }
            }
        }
    }

    fn fire_animation_events(
        &mut self,
        engine: &mut Engine,
        input: &InputState,
        audio_system: &mut AudioSystem,
        scene_manager: &mut SceneManager,
        last_step: &FrameStep,
    ) {
        for (entity, state_name) in last_step.animation_ended.clone() {
            let Some(behavior) = engine.active_scene.behavior(entity).cloned() else {
                continue;
            };
            if !behavior.enabled {
                continue;
            }
            let ctx = script_context(engine, entity, input);
            match self
                .runtime
                .call_on_animation_end(&behavior.script_path, &ctx, &state_name)
            {
                Ok(output) => {
                    apply_script_output(engine, audio_system, scene_manager, entity, output)
                }
                Err(error) => eprintln!("[Script ERROR] {error}"),
            }
        }
    }

    fn fire_collision_events(
        &mut self,
        engine: &mut Engine,
        input: &InputState,
        audio_system: &mut AudioSystem,
        scene_manager: &mut SceneManager,
        last_step: &FrameStep,
    ) {
        for collision in last_step.solid_collisions.clone() {
            let entity = collision.entity;
            let Some(behavior) = engine.active_scene.behavior(entity).cloned() else {
                continue;
            };
            if !behavior.enabled {
                continue;
            }

            let (other_id, other_tag, normal_x, normal_y) = match collision.obstacle {
                SolidObstacle::Entity(other) => {
                    let other_tag = engine
                        .active_scene
                        .tag(other)
                        .map(|tag| tag.tag.clone())
                        .unwrap_or_default();
                    let normal_x = if collision.axis == CollisionAxis::X {
                        1.0_f64
                    } else {
                        0.0_f64
                    };
                    let normal_y = if collision.axis == CollisionAxis::Y {
                        1.0_f64
                    } else {
                        0.0_f64
                    };
                    (other.raw() as i64, other_tag, normal_x, normal_y)
                }
                SolidObstacle::Tile { .. } => {
                    let normal_x = if collision.axis == CollisionAxis::X {
                        1.0_f64
                    } else {
                        0.0_f64
                    };
                    let normal_y = if collision.axis == CollisionAxis::Y {
                        1.0_f64
                    } else {
                        0.0_f64
                    };
                    (-1_i64, "tile".to_string(), normal_x, normal_y)
                }
            };

            let ctx = script_context(engine, entity, input);
            match self.runtime.call_on_collision(
                &behavior.script_path,
                &ctx,
                other_id,
                &other_tag,
                normal_x,
                normal_y,
            ) {
                Ok(output) => {
                    apply_script_output(engine, audio_system, scene_manager, entity, output)
                }
                Err(error) => eprintln!("[Script ERROR] {error}"),
            }
        }
    }
}

fn apply_script_output(
    engine: &mut Engine,
    audio_system: &mut AudioSystem,
    scene_manager: &mut SceneManager,
    entity: EntityId,
    output: ScriptOutput,
) {
    if let Some(velocity) = engine.active_scene.velocity_mut(entity) {
        if let Some(x) = output.set_vel_x {
            velocity.linear.x = x;
        }
        if let Some(y) = output.set_vel_y {
            velocity.linear.y = y;
        }
    }
    if let Some(node) = engine.active_scene.node_mut(entity) {
        if let Some(x) = output.set_pos_x {
            node.transform.position.x = x;
        }
        if let Some(y) = output.set_pos_y {
            node.transform.position.y = y;
        }
    }
    if output.destroy_self {
        let _ = engine.active_scene.despawn_node(entity);
    }
    if let Some(path) = output.load_scene {
        scene_manager.load_scene(path);
    }
    if let Some(state) = output.set_anim_state {
        if let Some(animation) = engine.active_scene.animation_mut(entity) {
            animation.set_state(&state);
            animation.playing = true;
        }
    }
    if let Some(clip) = output.play_audio {
        audio_system.play_clip_once(&clip, 1.0, false);
    }
    if let Some(text) = output.set_text {
        if let Some(world_text) = engine.active_scene.world_text_mut(entity) {
            world_text.text = text.clone();
        }
        if let Some(label) = engine.active_scene.ui_label_mut(entity) {
            label.text = text;
        }
    }
}

fn script_context(engine: &Engine, entity: EntityId, input: &InputState) -> ScriptContext {
    let node = engine.active_scene.node(entity);
    let velocity = engine.active_scene.velocity(entity);
    let anim_state = engine
        .active_scene
        .animation(entity)
        .map(|animation| animation.current_state.clone())
        .unwrap_or_default();
    let (mouse_world_x, mouse_world_y) = {
        let cursor = input.cursor_position().unwrap_or((0.0, 0.0));
        (cursor.0, cursor.1)
    };
    let (scroll_x, scroll_y) = input.scroll_delta();

    ScriptContext {
        entity_id: entity.raw(),
        pos_x: node.map(|node| node.transform.position.x).unwrap_or(0.0),
        pos_y: node.map(|node| node.transform.position.y).unwrap_or(0.0),
        vel_x: velocity.map(|velocity| velocity.linear.x).unwrap_or(0.0),
        vel_y: velocity.map(|velocity| velocity.linear.y).unwrap_or(0.0),
        tag: engine
            .active_scene
            .tag(entity)
            .map(|tag| tag.tag.clone())
            .unwrap_or_default(),
        keys_pressed: input
            .pressed_keys()
            .map(key_name)
            .filter(|key| !key.is_empty())
            .collect(),
        anim_state,
        mouse_world_x,
        mouse_world_y,
        mouse_left: input.is_mouse_down(MouseButton::Left),
        mouse_right: input.is_mouse_down(MouseButton::Right),
        mouse_middle: input.is_mouse_down(MouseButton::Middle),
        scroll_x,
        scroll_y,
    }
}
