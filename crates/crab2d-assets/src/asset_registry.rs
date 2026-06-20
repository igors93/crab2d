use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use crate::{AssetId, AssetKind, AssetRecord};

#[derive(Debug, Default)]
pub struct AssetRegistry {
    next_id: u64,
    records: BTreeMap<AssetId, AssetRecord>,
}

impl AssetRegistry {
    pub fn register(&mut self, kind: AssetKind, source: impl Into<PathBuf>) -> AssetId {
        self.try_register(kind, source)
            .expect("asset id space was exhausted")
    }

    pub fn try_register(
        &mut self,
        kind: AssetKind,
        source: impl Into<PathBuf>,
    ) -> Result<AssetId, AssetRegistryError> {
        let id = self.allocate_asset_id()?;
        self.records.insert(id, AssetRecord::new(id, kind, source));
        Ok(id)
    }

    pub fn get(&self, id: AssetId) -> Option<&AssetRecord> {
        self.records.get(&id)
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    fn allocate_asset_id(&mut self) -> Result<AssetId, AssetRegistryError> {
        let id = AssetId::from_raw(self.next_id);
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or(AssetRegistryError::AssetIdExhausted)?;
        Ok(id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetRegistryError {
    AssetIdExhausted,
}

impl fmt::Display for AssetRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AssetIdExhausted => formatter.write_str("asset id space was exhausted"),
        }
    }
}

impl Error for AssetRegistryError {}

#[cfg(test)]
mod tests {
    use crate::{AssetKind, AssetRegistry};

    #[test]
    fn registered_assets_receive_stable_ids() {
        let mut registry = AssetRegistry::default();

        let player = registry.register(AssetKind::Sprite, "sprites/player.png");
        let music = registry.register(AssetKind::Audio, "audio/theme.ogg");

        assert_eq!(player.raw(), 0);
        assert_eq!(music.raw(), 1);
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn registered_assets_can_be_read_back() {
        let mut registry = AssetRegistry::default();

        let id = registry.register(AssetKind::Tilemap, "maps/village.tmx");
        let record = registry.get(id).expect("asset should exist");

        assert_eq!(record.id, id);
        assert_eq!(record.kind, AssetKind::Tilemap);
        assert_eq!(record.source, std::path::PathBuf::from("maps/village.tmx"));
    }
}
