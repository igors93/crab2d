use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerComponent {
    pub name: String,
    pub once: bool,
}

impl TriggerComponent {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            once: false,
        }
    }

    pub fn once(mut self) -> Self {
        self.once = true;
        self
    }

    pub fn is_valid(&self) -> bool {
        !self.name.trim().is_empty()
    }
}
