use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BehaviorComponent {
    pub script_path: String,
    pub enabled: bool,
}

impl BehaviorComponent {
    pub fn new(script_path: impl Into<String>) -> Self {
        Self {
            script_path: script_path.into(),
            enabled: true,
        }
    }
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}
