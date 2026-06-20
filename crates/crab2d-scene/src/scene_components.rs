use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{Camera2DComponent, EntityId, SpriteComponent, TagComponent, TilemapComponent};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct SceneComponents {
    tags: BTreeMap<EntityId, TagComponent>,
    sprites: BTreeMap<EntityId, SpriteComponent>,
    cameras: BTreeMap<EntityId, Camera2DComponent>,
    tilemaps: BTreeMap<EntityId, TilemapComponent>,
}

impl SceneComponents {
    pub fn insert_tag(&mut self, entity: EntityId, component: TagComponent) {
        self.tags.insert(entity, component);
    }

    pub fn tag(&self, entity: EntityId) -> Option<&TagComponent> {
        self.tags.get(&entity)
    }

    pub fn remove_tag(&mut self, entity: EntityId) -> Option<TagComponent> {
        self.tags.remove(&entity)
    }

    pub fn insert_sprite(&mut self, entity: EntityId, component: SpriteComponent) {
        self.sprites.insert(entity, component);
    }

    pub fn sprite(&self, entity: EntityId) -> Option<&SpriteComponent> {
        self.sprites.get(&entity)
    }

    pub fn remove_sprite(&mut self, entity: EntityId) -> Option<SpriteComponent> {
        self.sprites.remove(&entity)
    }

    pub fn sprites(&self) -> impl Iterator<Item = (EntityId, &SpriteComponent)> {
        self.sprites
            .iter()
            .map(|(entity, component)| (*entity, component))
    }

    pub fn insert_camera(&mut self, entity: EntityId, component: Camera2DComponent) {
        self.cameras.insert(entity, component);
    }

    pub fn camera(&self, entity: EntityId) -> Option<&Camera2DComponent> {
        self.cameras.get(&entity)
    }

    pub fn insert_tilemap(&mut self, entity: EntityId, component: TilemapComponent) {
        self.tilemaps.insert(entity, component);
    }

    pub fn tilemap(&self, entity: EntityId) -> Option<&TilemapComponent> {
        self.tilemaps.get(&entity)
    }

    pub fn tilemap_mut(&mut self, entity: EntityId) -> Option<&mut TilemapComponent> {
        self.tilemaps.get_mut(&entity)
    }

    pub fn remove_tilemap(&mut self, entity: EntityId) -> Option<TilemapComponent> {
        self.tilemaps.remove(&entity)
    }

    pub fn tilemaps(&self) -> impl Iterator<Item = (EntityId, &TilemapComponent)> {
        self.tilemaps
            .iter()
            .map(|(entity, component)| (*entity, component))
    }

    pub fn remove_all(&mut self, entity: EntityId) {
        self.tags.remove(&entity);
        self.sprites.remove(&entity);
        self.cameras.remove(&entity);
        self.tilemaps.remove(&entity);
    }
}
