use std::path::PathBuf;

use crate::{AssetId, AssetKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetRecord {
    pub id: AssetId,
    pub kind: AssetKind,
    pub source: PathBuf,
}

impl AssetRecord {
    pub fn new(id: AssetId, kind: AssetKind, source: impl Into<PathBuf>) -> Self {
        Self {
            id,
            kind,
            source: source.into(),
        }
    }
}
