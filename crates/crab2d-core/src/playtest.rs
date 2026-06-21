use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PlaytestStatus {
    #[default]
    Stopped,
    Running,
    Paused,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeDiagnostic {
    pub kind: DiagnosticKind,
    pub message: String,
    pub entity_id: Option<u32>,
    pub script_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticKind {
    ScriptError,
    MissingAsset,
    SceneLoadError,
    Warning,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlaytestState {
    pub status: PlaytestStatus,
    pub current_scene: String,
    pub entity_count: usize,
    pub fps: f32,
    pub diagnostics: Vec<RuntimeDiagnostic>,
}

impl PlaytestState {
    pub fn new(scene_name: impl Into<String>) -> Self {
        Self {
            current_scene: scene_name.into(),
            ..Default::default()
        }
    }

    pub fn push_error(
        &mut self,
        message: impl Into<String>,
        script_path: Option<String>,
        entity_id: Option<u32>,
    ) {
        self.diagnostics.push(RuntimeDiagnostic {
            kind: DiagnosticKind::ScriptError,
            message: message.into(),
            entity_id,
            script_path,
        });
    }

    pub fn push_missing_asset(&mut self, path: impl Into<String>) {
        self.diagnostics.push(RuntimeDiagnostic {
            kind: DiagnosticKind::MissingAsset,
            message: path.into(),
            entity_id: None,
            script_path: None,
        });
    }

    pub fn clear_diagnostics(&mut self) {
        self.diagnostics.clear();
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| {
            d.kind == DiagnosticKind::ScriptError || d.kind == DiagnosticKind::SceneLoadError
        })
    }

    pub fn start(&mut self) {
        self.status = PlaytestStatus::Running;
        self.clear_diagnostics();
    }

    pub fn pause(&mut self) {
        if self.status == PlaytestStatus::Running {
            self.status = PlaytestStatus::Paused;
        }
    }

    pub fn stop(&mut self) {
        self.status = PlaytestStatus::Stopped;
    }

    pub fn resume(&mut self) {
        if self.status == PlaytestStatus::Paused {
            self.status = PlaytestStatus::Running;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_changes_status_to_running_and_clears_diagnostics() {
        let mut state = PlaytestState::new("Main Scene");
        state.diagnostics.push(RuntimeDiagnostic {
            kind: DiagnosticKind::Warning,
            message: "stale warning".to_string(),
            entity_id: None,
            script_path: None,
        });

        state.start();

        assert_eq!(state.status, PlaytestStatus::Running);
        assert!(state.diagnostics.is_empty());
    }

    #[test]
    fn push_error_adds_script_error_to_diagnostics() {
        let mut state = PlaytestState::new("Main Scene");
        state.push_error(
            "nil pointer",
            Some("scripts/player.rhai".to_string()),
            Some(3),
        );

        assert_eq!(state.diagnostics.len(), 1);
        assert_eq!(state.diagnostics[0].kind, DiagnosticKind::ScriptError);
        assert_eq!(state.diagnostics[0].message, "nil pointer");
        assert_eq!(state.diagnostics[0].entity_id, Some(3));
        assert_eq!(
            state.diagnostics[0].script_path,
            Some("scripts/player.rhai".to_string())
        );
    }

    #[test]
    fn has_errors_returns_true_when_script_error_present() {
        let mut state = PlaytestState::new("Main Scene");
        assert!(!state.has_errors());

        state.push_missing_asset("sprites/missing.png");
        assert!(!state.has_errors(), "missing asset is not an error");

        state.push_error("crash", None, None);
        assert!(state.has_errors());
    }

    #[test]
    fn pause_and_resume_only_work_in_correct_sequence() {
        let mut state = PlaytestState::new("Main Scene");

        // Cannot pause when stopped
        state.pause();
        assert_eq!(state.status, PlaytestStatus::Stopped);

        // Cannot resume when stopped
        state.resume();
        assert_eq!(state.status, PlaytestStatus::Stopped);

        state.start();
        assert_eq!(state.status, PlaytestStatus::Running);

        state.pause();
        assert_eq!(state.status, PlaytestStatus::Paused);

        // Cannot pause again when already paused
        state.pause();
        assert_eq!(state.status, PlaytestStatus::Paused);

        state.resume();
        assert_eq!(state.status, PlaytestStatus::Running);

        state.stop();
        assert_eq!(state.status, PlaytestStatus::Stopped);
    }
}
