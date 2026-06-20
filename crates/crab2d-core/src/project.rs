use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub root: Option<PathBuf>,
    pub metadata: ProjectMetadata,
}

impl ProjectInfo {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            root: None,
            metadata: ProjectMetadata::default(),
        }
    }

    pub fn untitled() -> Self {
        Self::new("Untitled Crab2D Project")
    }

    pub fn with_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.root = Some(root.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub engine_version: String,
    pub philosophy_version: u32,
}

impl Default for ProjectMetadata {
    fn default() -> Self {
        Self {
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            philosophy_version: 1,
        }
    }
}
