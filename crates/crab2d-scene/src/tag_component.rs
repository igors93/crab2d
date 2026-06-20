use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagComponent {
    pub tag: String,
}

impl TagComponent {
    pub fn new(tag: impl Into<String>) -> Self {
        Self { tag: tag.into() }
    }

    pub fn is_empty(&self) -> bool {
        self.tag.is_empty()
    }
}
