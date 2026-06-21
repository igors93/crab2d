use rhai::{Engine, EvalAltResult, Scope, AST};
use std::collections::HashMap;

pub struct ScriptRuntime {
    engine: Engine,
    scripts: HashMap<String, AST>,
}

#[derive(Debug, Clone, Default)]
pub struct ScriptContext {
    pub entity_id: u64,
    pub pos_x: f32,
    pub pos_y: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub tag: String,
    pub keys_pressed: Vec<String>,
    /// Current animation state name
    pub anim_state: String,
    /// Mouse cursor position in world space
    pub mouse_world_x: f32,
    pub mouse_world_y: f32,
    /// Mouse buttons held this frame
    pub mouse_left: bool,
    pub mouse_right: bool,
    pub mouse_middle: bool,
    /// Mouse scroll delta this frame
    pub scroll_x: f32,
    pub scroll_y: f32,
}

#[derive(Debug, Clone, Default)]
pub struct ScriptOutput {
    pub set_vel_x: Option<f32>,
    pub set_vel_y: Option<f32>,
    pub set_pos_x: Option<f32>,
    pub set_pos_y: Option<f32>,
    pub destroy_self: bool,
    pub load_scene: Option<String>,
    /// Switch animation state
    pub set_anim_state: Option<String>,
    /// Play an audio clip by path
    pub play_audio: Option<String>,
    /// Update a WorldText or UiLabel text on the same entity
    pub set_text: Option<String>,
}

impl ScriptRuntime {
    pub fn new() -> Self {
        let mut engine = Engine::new();
        engine.set_max_operations(500_000);
        Self {
            engine,
            scripts: HashMap::new(),
        }
    }

    pub fn load_script(&mut self, path: &str, source: &str) -> Result<(), String> {
        match self.engine.compile(source) {
            Ok(ast) => {
                self.scripts.insert(path.to_string(), ast);
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn is_loaded(&self, path: &str) -> bool {
        self.scripts.contains_key(path)
    }

    pub fn call_on_start(
        &self,
        script_path: &str,
        ctx: &ScriptContext,
    ) -> Result<ScriptOutput, String> {
        let Some(ast) = self.scripts.get(script_path) else {
            return Ok(ScriptOutput::default());
        };
        let mut scope = make_scope(ctx);
        if let Err(e) = self.engine.call_fn::<()>(&mut scope, ast, "on_start", ()) {
            if !is_fn_not_found(&e) {
                return Err(format!("[{}] on_start: {e}", script_path));
            }
        }
        Ok(read_output(&scope))
    }

    pub fn call_on_update(
        &self,
        script_path: &str,
        ctx: &ScriptContext,
        dt: f32,
    ) -> Result<ScriptOutput, String> {
        let Some(ast) = self.scripts.get(script_path) else {
            return Ok(ScriptOutput::default());
        };
        let mut scope = make_scope(ctx);
        scope.push("dt", dt as f64);
        if let Err(e) = self
            .engine
            .call_fn::<()>(&mut scope, ast, "on_update", (dt as f64,))
        {
            if !is_fn_not_found(&e) {
                return Err(format!("[{}] on_update: {e}", script_path));
            }
        }
        Ok(read_output(&scope))
    }

    pub fn call_on_trigger(
        &self,
        script_path: &str,
        ctx: &ScriptContext,
        name: &str,
    ) -> Result<ScriptOutput, String> {
        let Some(ast) = self.scripts.get(script_path) else {
            return Ok(ScriptOutput::default());
        };
        let mut scope = make_scope(ctx);
        if let Err(e) =
            self.engine
                .call_fn::<()>(&mut scope, ast, "on_trigger", (name.to_string(),))
        {
            if !is_fn_not_found(&e) {
                return Err(format!("[{}] on_trigger('{name}'): {e}", script_path));
            }
        }
        Ok(read_output(&scope))
    }

    pub fn call_on_animation_end(
        &self,
        script_path: &str,
        ctx: &ScriptContext,
        state_name: &str,
    ) -> Result<ScriptOutput, String> {
        let Some(ast) = self.scripts.get(script_path) else {
            return Ok(ScriptOutput::default());
        };
        let mut scope = make_scope(ctx);
        if let Err(e) = self.engine.call_fn::<()>(
            &mut scope,
            ast,
            "on_animation_end",
            (state_name.to_string(),),
        ) {
            if !is_fn_not_found(&e) {
                return Err(format!(
                    "[{}] on_animation_end('{state_name}'): {e}",
                    script_path
                ));
            }
        }
        Ok(read_output(&scope))
    }

    pub fn call_on_collision(
        &self,
        script_path: &str,
        ctx: &ScriptContext,
        other_id: i64,
        other_tag: &str,
        normal_x: f64,
        normal_y: f64,
    ) -> Result<ScriptOutput, String> {
        let Some(ast) = self.scripts.get(script_path) else {
            return Ok(ScriptOutput::default());
        };
        let mut scope = make_scope(ctx);
        if let Err(e) = self.engine.call_fn::<()>(
            &mut scope,
            ast,
            "on_collision",
            (other_id, other_tag.to_string(), normal_x, normal_y),
        ) {
            if !is_fn_not_found(&e) {
                return Err(format!("[{}] on_collision: {e}", script_path));
            }
        }
        Ok(read_output(&scope))
    }

    pub fn unload_script(&mut self, path: &str) {
        self.scripts.remove(path);
    }

    pub fn unload_all(&mut self) {
        self.scripts.clear();
    }
}

impl Default for ScriptRuntime {
    fn default() -> Self {
        Self::new()
    }
}

fn make_scope(ctx: &ScriptContext) -> Scope<'static> {
    let mut s = Scope::new();
    // Input context
    s.push("entity_id", ctx.entity_id as i64);
    s.push("pos_x", ctx.pos_x as f64);
    s.push("pos_y", ctx.pos_y as f64);
    s.push("vel_x", ctx.vel_x as f64);
    s.push("vel_y", ctx.vel_y as f64);
    s.push("tag", ctx.tag.clone());
    s.push("anim_state", ctx.anim_state.clone());
    s.push("mouse_world_x", ctx.mouse_world_x as f64);
    s.push("mouse_world_y", ctx.mouse_world_y as f64);
    s.push("mouse_left", ctx.mouse_left);
    s.push("mouse_right", ctx.mouse_right);
    s.push("mouse_middle", ctx.mouse_middle);
    s.push("scroll_x", ctx.scroll_x as f64);
    s.push("scroll_y", ctx.scroll_y as f64);
    // Output vars — scripts assign these to communicate back
    s.push("set_vel_x", rhai::Dynamic::UNIT);
    s.push("set_vel_y", rhai::Dynamic::UNIT);
    s.push("set_pos_x", rhai::Dynamic::UNIT);
    s.push("set_pos_y", rhai::Dynamic::UNIT);
    s.push("destroy", false);
    s.push("load_scene", rhai::Dynamic::UNIT);
    s.push("set_anim_state", rhai::Dynamic::UNIT);
    s.push("play_audio", rhai::Dynamic::UNIT);
    s.push("set_text", rhai::Dynamic::UNIT);
    s
}

fn is_fn_not_found(err: &EvalAltResult) -> bool {
    matches!(err, EvalAltResult::ErrorFunctionNotFound(_, _))
}

fn read_output(scope: &Scope) -> ScriptOutput {
    let mut out = ScriptOutput::default();
    if let Some(v) = scope.get_value::<f64>("set_vel_x") {
        out.set_vel_x = Some(v as f32);
    }
    if let Some(v) = scope.get_value::<f64>("set_vel_y") {
        out.set_vel_y = Some(v as f32);
    }
    if let Some(v) = scope.get_value::<f64>("set_pos_x") {
        out.set_pos_x = Some(v as f32);
    }
    if let Some(v) = scope.get_value::<f64>("set_pos_y") {
        out.set_pos_y = Some(v as f32);
    }
    if let Some(v) = scope.get_value::<bool>("destroy") {
        out.destroy_self = v;
    }
    if let Some(v) = scope.get_value::<String>("load_scene") {
        out.load_scene = Some(v);
    }
    if let Some(v) = scope.get_value::<String>("set_anim_state") {
        out.set_anim_state = Some(v);
    }
    if let Some(v) = scope.get_value::<String>("play_audio") {
        out.play_audio = Some(v);
    }
    if let Some(v) = scope.get_value::<String>("set_text") {
        out.set_text = Some(v);
    }
    out
}
