use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetId(u64);

impl AssetId {
    pub fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetKind {
    Sprite,
    Tilemap,
    Audio,
    Script,
    Config,
}

#[derive(Debug, Clone)]
pub struct AssetRecord {
    pub id: AssetId,
    pub kind: AssetKind,
    pub source: PathBuf,
}

#[derive(Debug, Default)]
pub struct AssetRegistry {
    next_id: u64,
    records: BTreeMap<AssetId, AssetRecord>,
}

impl AssetRegistry {
    pub fn register(&mut self, kind: AssetKind, source: impl Into<PathBuf>) -> AssetId {
        let id = AssetId(self.next_id);
        self.next_id += 1;
        self.records.insert(
            id,
            AssetRecord {
                id,
                kind,
                source: source.into(),
            },
        );
        id
    }

    pub fn get(&self, id: AssetId) -> Option<&AssetRecord> {
        self.records.get(&id)
    }
}
