use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::asset_markers::{AssetType, Audio, Config, Script, Sprite, Tilemap};
use crate::typed_id::TypedAssetId;
use crate::{AssetId, AssetKind, AssetRecord};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetRegistry {
    next_id: u64,
    records: BTreeMap<AssetId, AssetRecord>,
}

impl AssetRegistry {
    pub fn register(&mut self, kind: AssetKind, source: impl Into<PathBuf>) -> AssetId {
        self.try_register(kind, source)
            .unwrap_or_else(|e| panic!("{e}"))
    }

    pub fn try_register(
        &mut self,
        kind: AssetKind,
        source: impl Into<PathBuf>,
    ) -> Result<AssetId, AssetRegistryError> {
        let source = source.into();
        if source.as_os_str().is_empty() {
            return Err(AssetRegistryError::EmptySourcePath);
        }
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

    // ── Typed helpers ────────────────────────────────────────────────────────

    /// Registers an asset and returns a [`TypedAssetId`] branded with `K`.
    ///
    /// Panics if the id space is exhausted (requires registering 2^64 assets).
    pub fn register_typed<K: AssetType>(&mut self, source: impl Into<PathBuf>) -> TypedAssetId<K> {
        self.try_register_typed(source)
            .unwrap_or_else(|e| panic!("{e}"))
    }

    /// Fallible variant of [`register_typed`](Self::register_typed).
    pub fn try_register_typed<K: AssetType>(
        &mut self,
        source: impl Into<PathBuf>,
    ) -> Result<TypedAssetId<K>, AssetRegistryError> {
        let id = self.try_register(K::kind(), source)?;
        Ok(TypedAssetId::new(id))
    }

    /// Looks up an asset by its typed id.
    pub fn get_typed<K: AssetType>(&self, id: TypedAssetId<K>) -> Option<&AssetRecord> {
        self.get(id.id())
    }

    // Per-kind convenience wrappers — thin delegates to `register_typed`.

    pub fn register_sprite(&mut self, source: impl Into<PathBuf>) -> TypedAssetId<Sprite> {
        self.register_typed(source)
    }

    pub fn try_register_sprite(
        &mut self,
        source: impl Into<PathBuf>,
    ) -> Result<TypedAssetId<Sprite>, AssetRegistryError> {
        self.try_register_typed(source)
    }

    pub fn register_tilemap(&mut self, source: impl Into<PathBuf>) -> TypedAssetId<Tilemap> {
        self.register_typed(source)
    }

    pub fn try_register_tilemap(
        &mut self,
        source: impl Into<PathBuf>,
    ) -> Result<TypedAssetId<Tilemap>, AssetRegistryError> {
        self.try_register_typed(source)
    }

    pub fn register_audio(&mut self, source: impl Into<PathBuf>) -> TypedAssetId<Audio> {
        self.register_typed(source)
    }

    pub fn try_register_audio(
        &mut self,
        source: impl Into<PathBuf>,
    ) -> Result<TypedAssetId<Audio>, AssetRegistryError> {
        self.try_register_typed(source)
    }

    pub fn register_script(&mut self, source: impl Into<PathBuf>) -> TypedAssetId<Script> {
        self.register_typed(source)
    }

    pub fn try_register_script(
        &mut self,
        source: impl Into<PathBuf>,
    ) -> Result<TypedAssetId<Script>, AssetRegistryError> {
        self.try_register_typed(source)
    }

    pub fn register_config(&mut self, source: impl Into<PathBuf>) -> TypedAssetId<Config> {
        self.register_typed(source)
    }

    pub fn try_register_config(
        &mut self,
        source: impl Into<PathBuf>,
    ) -> Result<TypedAssetId<Config>, AssetRegistryError> {
        self.try_register_typed(source)
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
    EmptySourcePath,
}

impl fmt::Display for AssetRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AssetIdExhausted => formatter.write_str("asset id space was exhausted"),
            Self::EmptySourcePath => formatter.write_str("asset source path cannot be empty"),
        }
    }
}

impl Error for AssetRegistryError {}

#[cfg(test)]
mod tests {
    use crate::asset_markers::Sprite;
    use crate::{AssetKind, AssetRegistry, AssetRegistryError};

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

    #[test]
    fn typed_helpers_set_correct_kind() {
        let mut registry = AssetRegistry::default();

        let sprite = registry.register_sprite("sprites/hero.png");
        let audio = registry.register_audio("audio/bgm.ogg");
        let tilemap = registry.register_tilemap("maps/level1.tmx");
        let script = registry.register_script("scripts/ai.lua");
        let config = registry.register_config("config/settings.toml");

        assert_eq!(registry.get_typed(sprite).unwrap().kind, AssetKind::Sprite);
        assert_eq!(registry.get_typed(audio).unwrap().kind, AssetKind::Audio);
        assert_eq!(
            registry.get_typed(tilemap).unwrap().kind,
            AssetKind::Tilemap
        );
        assert_eq!(registry.get_typed(script).unwrap().kind, AssetKind::Script);
        assert_eq!(registry.get_typed(config).unwrap().kind, AssetKind::Config);
    }

    #[test]
    fn typed_id_and_untyped_id_refer_to_same_record() {
        let mut registry = AssetRegistry::default();

        let typed = registry.register_sprite("sprites/player.png");
        let untyped = typed.id();

        assert_eq!(
            registry.get_typed(typed).unwrap().id,
            registry.get(untyped).unwrap().id,
        );
    }

    #[test]
    fn try_register_typed_returns_ok_on_success() {
        let mut registry = AssetRegistry::default();

        let result = registry.try_register_typed::<Sprite>("sprites/enemy.png");

        assert!(result.is_ok());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn empty_source_path_is_rejected() {
        let mut registry = AssetRegistry::default();

        let result = registry.try_register(AssetKind::Sprite, "");

        assert_eq!(result, Err(AssetRegistryError::EmptySourcePath));
        assert!(registry.is_empty());
    }

    #[test]
    fn empty_source_path_is_rejected_via_typed_helper() {
        let mut registry = AssetRegistry::default();

        let result = registry.try_register_sprite("");

        assert_eq!(result, Err(AssetRegistryError::EmptySourcePath));
        assert!(registry.is_empty());
    }

    #[test]
    fn per_kind_try_helpers_return_typed_ids() {
        let mut registry = AssetRegistry::default();

        let sprite = registry
            .try_register_sprite("sprites/hero.png")
            .expect("sprite should register");
        let audio = registry
            .try_register_audio("audio/bgm.ogg")
            .expect("audio should register");
        let tilemap = registry
            .try_register_tilemap("maps/level1.tmx")
            .expect("tilemap should register");
        let script = registry
            .try_register_script("scripts/ai.lua")
            .expect("script should register");
        let config = registry
            .try_register_config("config/settings.toml")
            .expect("config should register");

        assert_eq!(registry.get_typed(sprite).unwrap().kind, AssetKind::Sprite);
        assert_eq!(registry.get_typed(audio).unwrap().kind, AssetKind::Audio);
        assert_eq!(
            registry.get_typed(tilemap).unwrap().kind,
            AssetKind::Tilemap
        );
        assert_eq!(registry.get_typed(script).unwrap().kind, AssetKind::Script);
        assert_eq!(registry.get_typed(config).unwrap().kind, AssetKind::Config);
    }

    #[test]
    #[should_panic(expected = "asset source path cannot be empty")]
    fn register_panics_on_empty_source() {
        let mut registry = AssetRegistry::default();
        registry.register(AssetKind::Audio, "");
    }

    #[test]
    fn typed_ids_are_ordered_by_registration_sequence() {
        let mut registry = AssetRegistry::default();

        let first = registry.register_sprite("sprites/a.png");
        let second = registry.register_sprite("sprites/b.png");

        assert!(first < second);
    }
}
