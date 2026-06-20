use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    Camera2DComponent, Collider2DComponent, EntityId, SpriteComponent, TagComponent,
    TilemapComponent, Velocity2DComponent,
};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct SceneComponents {
    #[serde(default)]
    tags: BTreeMap<EntityId, TagComponent>,
    #[serde(default)]
    sprites: BTreeMap<EntityId, SpriteComponent>,
    #[serde(default)]
    cameras: BTreeMap<EntityId, Camera2DComponent>,
    #[serde(default)]
    tilemaps: BTreeMap<EntityId, TilemapComponent>,
    #[serde(default)]
    velocities: BTreeMap<EntityId, Velocity2DComponent>,
    #[serde(default)]
    colliders: BTreeMap<EntityId, Collider2DComponent>,
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

    pub fn insert_velocity(&mut self, entity: EntityId, component: Velocity2DComponent) {
        self.velocities.insert(entity, component);
    }

    pub fn velocity(&self, entity: EntityId) -> Option<&Velocity2DComponent> {
        self.velocities.get(&entity)
    }

    pub fn velocity_mut(&mut self, entity: EntityId) -> Option<&mut Velocity2DComponent> {
        self.velocities.get_mut(&entity)
    }

    pub fn remove_velocity(&mut self, entity: EntityId) -> Option<Velocity2DComponent> {
        self.velocities.remove(&entity)
    }

    pub fn velocities(&self) -> impl Iterator<Item = (EntityId, &Velocity2DComponent)> {
        self.velocities
            .iter()
            .map(|(entity, component)| (*entity, component))
    }

    pub fn insert_collider(&mut self, entity: EntityId, component: Collider2DComponent) {
        self.colliders.insert(entity, component);
    }

    pub fn collider(&self, entity: EntityId) -> Option<&Collider2DComponent> {
        self.colliders.get(&entity)
    }

    pub fn remove_collider(&mut self, entity: EntityId) -> Option<Collider2DComponent> {
        self.colliders.remove(&entity)
    }

    pub fn colliders(&self) -> impl Iterator<Item = (EntityId, &Collider2DComponent)> {
        self.colliders
            .iter()
            .map(|(entity, component)| (*entity, component))
    }

    pub fn remove_all(&mut self, entity: EntityId) {
        self.tags.remove(&entity);
        self.sprites.remove(&entity);
        self.cameras.remove(&entity);
        self.tilemaps.remove(&entity);
        self.velocities.remove(&entity);
        self.colliders.remove(&entity);
    }
}
