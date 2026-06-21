//! Integration tests ensuring script errors surface as Result::Err instead of being swallowed.

#[cfg(test)]
mod tests {
    use crab2d_core::script_runtime::{ScriptContext, ScriptRuntime};

    fn default_ctx() -> ScriptContext {
        ScriptContext {
            entity_id: 1,
            tag: "player".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn load_script_with_syntax_error_returns_err() {
        let mut runtime = ScriptRuntime::new();
        let result =
            runtime.load_script("bad_script", "fn on_update(dt) { this is not valid rhai }");
        assert!(result.is_err(), "syntax error should return Err");
    }

    #[test]
    fn on_update_runtime_error_returns_err() {
        let mut runtime = ScriptRuntime::new();
        // This script compiles fine but panics at runtime (division by zero in Rhai = error).
        runtime
            .load_script("div_zero", "fn on_update(dt) { let x = 1 / 0; }")
            .expect("script should compile");

        let ctx = default_ctx();
        let result = runtime.call_on_update("div_zero", &ctx, 0.016);
        assert!(
            result.is_err(),
            "runtime error in on_update should return Err, got {:?}",
            result
        );
    }

    #[test]
    fn on_start_runtime_error_returns_err() {
        let mut runtime = ScriptRuntime::new();
        runtime
            .load_script(
                "bad_start",
                "fn on_start() { throw \"intentional error\"; }",
            )
            .expect("script compiles");

        let ctx = default_ctx();
        let result = runtime.call_on_start("bad_start", &ctx);
        assert!(
            result.is_err(),
            "thrown error in on_start should surface as Err"
        );
        let message = result.unwrap_err();
        assert!(
            message.contains("bad_start"),
            "error message should include the script path, got: {message}"
        );
    }

    #[test]
    fn on_trigger_runtime_error_returns_err() {
        let mut runtime = ScriptRuntime::new();
        runtime
            .load_script("bad_trigger", "fn on_trigger(name) { throw name; }")
            .expect("script compiles");

        let ctx = default_ctx();
        let result = runtime.call_on_trigger("bad_trigger", &ctx, "coin");
        assert!(
            result.is_err(),
            "thrown error in on_trigger should surface as Err"
        );
    }

    #[test]
    fn valid_script_returns_ok_with_output() {
        let mut runtime = ScriptRuntime::new();
        runtime
            .load_script("good_script", "fn on_update(dt) { set_vel_x = 42.0; }")
            .expect("script compiles");

        let ctx = default_ctx();
        let result = runtime.call_on_update("good_script", &ctx, 0.016);
        assert!(result.is_ok(), "valid script should return Ok");
        let output = result.unwrap();
        assert_eq!(output.set_vel_x, Some(42.0));
    }

    #[test]
    fn missing_function_is_not_an_error() {
        let mut runtime = ScriptRuntime::new();
        // Script exists but doesn't define on_start.
        runtime
            .load_script("no_on_start", "fn on_update(dt) {}")
            .expect("script compiles");

        let ctx = default_ctx();
        // Calling a missing function should return Ok with default output (Rhai's call_fn
        // treats a missing function as a no-op when using the lenient variant).
        let result = runtime.call_on_start("no_on_start", &ctx);
        assert!(
            result.is_ok(),
            "missing on_start should not be a hard error, got {:?}",
            result
        );
    }

    #[test]
    fn unload_all_clears_cached_scripts() {
        let mut runtime = ScriptRuntime::new();
        runtime
            .load_script("cached", "fn on_update(dt) {}")
            .expect("loads");
        assert!(runtime.is_loaded("cached"));

        runtime.unload_all();
        assert!(
            !runtime.is_loaded("cached"),
            "script should be gone after unload_all"
        );
    }
}
