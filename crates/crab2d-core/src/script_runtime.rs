use rhai::{Engine, Scope, AST};
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
}

#[derive(Debug, Clone, Default)]
pub struct ScriptOutput {
    pub set_vel_x: Option<f32>,
    pub set_vel_y: Option<f32>,
    pub set_pos_x: Option<f32>,
    pub set_pos_y: Option<f32>,
    pub destroy_self: bool,
    pub load_scene: Option<String>,
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

    pub fn call_on_start(&self, script_path: &str, ctx: &ScriptContext) -> ScriptOutput {
        let Some(ast) = self.scripts.get(script_path) else {
            return ScriptOutput::default();
        };
        let mut scope = make_scope(ctx);
        let _ = self.engine.call_fn::<()>(&mut scope, ast, "on_start", ());
        read_output(&scope)
    }

    pub fn call_on_update(&self, script_path: &str, ctx: &ScriptContext, dt: f32) -> ScriptOutput {
        let Some(ast) = self.scripts.get(script_path) else {
            return ScriptOutput::default();
        };
        let mut scope = make_scope(ctx);
        scope.push("dt", dt as f64);
        let _ = self
            .engine
            .call_fn::<()>(&mut scope, ast, "on_update", (dt as f64,));
        read_output(&scope)
    }

    pub fn call_on_trigger(
        &self,
        script_path: &str,
        ctx: &ScriptContext,
        name: &str,
    ) -> ScriptOutput {
        let Some(ast) = self.scripts.get(script_path) else {
            return ScriptOutput::default();
        };
        let mut scope = make_scope(ctx);
        let _ = self
            .engine
            .call_fn::<()>(&mut scope, ast, "on_trigger", (name.to_string(),));
        read_output(&scope)
    }
}

impl Default for ScriptRuntime {
    fn default() -> Self {
        Self::new()
    }
}

fn make_scope(ctx: &ScriptContext) -> Scope<'static> {
    let mut s = Scope::new();
    s.push("entity_id", ctx.entity_id as i64);
    s.push("pos_x", ctx.pos_x as f64);
    s.push("pos_y", ctx.pos_y as f64);
    s.push("vel_x", ctx.vel_x as f64);
    s.push("vel_y", ctx.vel_y as f64);
    s.push("tag", ctx.tag.clone());
    // Output vars — scripts assign these to communicate back
    s.push("set_vel_x", rhai::Dynamic::UNIT);
    s.push("set_vel_y", rhai::Dynamic::UNIT);
    s.push("set_pos_x", rhai::Dynamic::UNIT);
    s.push("set_pos_y", rhai::Dynamic::UNIT);
    s.push("destroy", false);
    s.push("load_scene", rhai::Dynamic::UNIT);
    s
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
    out
}
