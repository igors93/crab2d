use std::error::Error;
use std::fmt;

use crate::scene_components::SceneComponents;
use crate::{Camera2DComponent, EntityId, Node2D, SpriteComponent, TagComponent, Transform2D};

#[derive(Debug, Clone)]
pub struct Scene {
    pub name: String,
    next_id: u64,
    nodes: Vec<Node2D>,
    components: SceneComponents,
}

impl Scene {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            next_id: 0,
            nodes: Vec::new(),
            components: SceneComponents::default(),
        }
    }

    pub fn spawn_node(&mut self, name: impl Into<String>) -> EntityId {
        self.try_spawn_node(name)
            .expect("scene entity id space was exhausted")
    }

    pub fn spawn_node_with_transform(
        &mut self,
        name: impl Into<String>,
        transform: Transform2D,
    ) -> Result<EntityId, SceneError> {
        if !transform.is_finite() {
            return Err(SceneError::InvalidTransform);
        }

        let id = self.allocate_entity_id()?;
        self.nodes
            .push(Node2D::new(id, name).with_transform(transform));
        Ok(id)
    }

    pub fn try_spawn_node(&mut self, name: impl Into<String>) -> Result<EntityId, SceneError> {
        let id = self.allocate_entity_id()?;
        self.nodes.push(Node2D::new(id, name));
        Ok(id)
    }

    pub fn nodes(&self) -> &[Node2D] {
        &self.nodes
    }

    pub fn add_tag(&mut self, entity: EntityId, component: TagComponent) -> Result<(), SceneError> {
        self.ensure_entity_exists(entity)?;
        if component.is_empty() {
            return Err(SceneError::EmptyTag);
        }
        self.components.insert_tag(entity, component);
        Ok(())
    }

    pub fn tag(&self, entity: EntityId) -> Option<&TagComponent> {
        self.components.tag(entity)
    }

    pub fn add_sprite(
        &mut self,
        entity: EntityId,
        component: SpriteComponent,
    ) -> Result<(), SceneError> {
        self.ensure_entity_exists(entity)?;
        if component.asset_path.is_empty() {
            return Err(SceneError::EmptyAssetPath);
        }
        self.components.insert_sprite(entity, component);
        Ok(())
    }

    pub fn sprite(&self, entity: EntityId) -> Option<&SpriteComponent> {
        self.components.sprite(entity)
    }

    pub fn sprites(&self) -> impl Iterator<Item = (EntityId, &SpriteComponent)> {
        self.components.sprites()
    }

    pub fn add_camera(
        &mut self,
        entity: EntityId,
        component: Camera2DComponent,
    ) -> Result<(), SceneError> {
        self.ensure_entity_exists(entity)?;
        if !component.zoom.is_finite() || component.zoom <= 0.0 {
            return Err(SceneError::InvalidCameraZoom);
        }
        self.components.insert_camera(entity, component);
        Ok(())
    }

    pub fn camera(&self, entity: EntityId) -> Option<&Camera2DComponent> {
        self.components.camera(entity)
    }

    pub fn node(&self, id: EntityId) -> Option<&Node2D> {
        self.nodes.iter().find(|node| node.id == id)
    }

    pub fn node_mut(&mut self, id: EntityId) -> Option<&mut Node2D> {
        self.nodes.iter_mut().find(|node| node.id == id)
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    fn allocate_entity_id(&mut self) -> Result<EntityId, SceneError> {
        let id = EntityId::from_raw(self.next_id);
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or(SceneError::EntityIdExhausted)?;
        Ok(id)
    }

    fn ensure_entity_exists(&self, entity: EntityId) -> Result<(), SceneError> {
        if self.node(entity).is_some() {
            Ok(())
        } else {
            Err(SceneError::EntityNotFound)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneError {
    EntityIdExhausted,
    EntityNotFound,
    EmptyAssetPath,
    EmptyTag,
    InvalidCameraZoom,
    InvalidTransform,
}

impl fmt::Display for SceneError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EntityIdExhausted => formatter.write_str("scene entity id space was exhausted"),
            Self::EntityNotFound => formatter.write_str("scene entity was not found"),
            Self::EmptyAssetPath => formatter.write_str("sprite asset path cannot be empty"),
            Self::EmptyTag => formatter.write_str("tag cannot be empty"),
            Self::InvalidCameraZoom => {
                formatter.write_str("camera zoom must be finite and positive")
            }
            Self::InvalidTransform => formatter.write_str("transform contains non-finite values"),
        }
    }
}

impl Error for SceneError {}

#[cfg(test)]
mod tests {
    use crate::{
        Camera2DComponent, EntityId, Scene, SceneError, SpriteComponent, TagComponent, Transform2D,
        Vec2,
    };

    #[test]
    fn spawned_nodes_receive_stable_ids() {
        let mut scene = Scene::new("Test Scene");

        let player = scene.spawn_node("Player");
        let camera = scene.spawn_node("Camera2D");

        assert_eq!(player.raw(), 0);
        assert_eq!(camera.raw(), 1);
        assert_eq!(scene.len(), 2);
    }

    #[test]
    fn rejects_non_finite_transforms() {
        let mut scene = Scene::new("Test Scene");
        let transform = Transform2D::from_position(Vec2::new(f32::NAN, 0.0));

        let result = scene.spawn_node_with_transform("Broken Node", transform);

        assert!(result.is_err());
        assert!(scene.is_empty());
    }

    #[test]
    fn components_can_be_attached_to_existing_entities() {
        let mut scene = Scene::new("Test Scene");
        let player = scene.spawn_node("Player");
        let camera = scene.spawn_node("Camera2D");

        scene
            .add_tag(player, TagComponent::new("player"))
            .expect("tag should attach");
        scene
            .add_sprite(
                player,
                SpriteComponent::new("sprites/player.png").with_z_index(10),
            )
            .expect("sprite should attach");
        scene
            .add_camera(camera, Camera2DComponent::new().with_zoom(2.0))
            .expect("camera should attach");

        assert_eq!(scene.tag(player).expect("tag exists").tag, "player");
        assert_eq!(
            scene.sprite(player).expect("sprite exists").asset_path,
            "sprites/player.png"
        );
        assert_eq!(scene.camera(camera).expect("camera exists").zoom, 2.0);
    }

    #[test]
    fn components_reject_missing_entities() {
        let mut scene = Scene::new("Test Scene");
        let missing = EntityId::from_raw(999);

        let result = scene.add_sprite(missing, SpriteComponent::new("sprites/missing.png"));

        assert_eq!(result, Err(SceneError::EntityNotFound));
    }

    #[test]
    fn sprites_iterator_returns_attached_sprites() {
        let mut scene = Scene::new("Test Scene");
        let player = scene.spawn_node("Player");

        scene
            .add_sprite(player, SpriteComponent::new("sprites/player.png"))
            .expect("sprite should attach");

        let sprites: Vec<_> = scene.sprites().collect();

        assert_eq!(sprites.len(), 1);
        assert_eq!(sprites[0].0, player);
        assert_eq!(sprites[0].1.asset_path, "sprites/player.png");
    }
}
