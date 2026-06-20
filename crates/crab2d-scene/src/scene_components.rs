use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{Camera2DComponent, EntityId, SpriteComponent, TagComponent};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct SceneComponents {
    tags: BTreeMap<EntityId, TagComponent>,
    sprites: BTreeMap<EntityId, SpriteComponent>,
    cameras: BTreeMap<EntityId, Camera2DComponent>,
}

impl SceneComponents {
    pub fn insert_tag(&mut self, entity: EntityId, component: TagComponent) {
        self.tags.insert(entity, component);
    }

    pub fn tag(&self, entity: EntityId) -> Option<&TagComponent> {
        self.tags.get(&entity)
    }

    pub fn insert_sprite(&mut self, entity: EntityId, component: SpriteComponent) {
        self.sprites.insert(entity, component);
    }

    pub fn sprite(&self, entity: EntityId) -> Option<&SpriteComponent> {
        self.sprites.get(&entity)
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
}
