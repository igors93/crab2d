use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::scene_components::SceneComponents;
use crate::{
    Camera2DComponent, Collider2DComponent, EntityId, Node2D, SpriteComponent, TagComponent,
    TilemapComponent, TilemapError, Transform2D, Velocity2DComponent,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
        self.try_spawn_node(name).unwrap_or_else(|e| panic!("{e}"))
    }

    pub fn try_spawn_node(&mut self, name: impl Into<String>) -> Result<EntityId, SceneError> {
        let name = name.into();
        if name.is_empty() {
            return Err(SceneError::EmptyNodeName);
        }
        let id = self.allocate_entity_id()?;
        self.nodes.push(Node2D::new(id, name));
        Ok(id)
    }

    pub fn spawn_node_with_transform(
        &mut self,
        name: impl Into<String>,
        transform: Transform2D,
    ) -> Result<EntityId, SceneError> {
        let name = name.into();
        if name.is_empty() {
            return Err(SceneError::EmptyNodeName);
        }
        if !transform.is_finite() {
            return Err(SceneError::InvalidTransform);
        }
        let id = self.allocate_entity_id()?;
        self.nodes
            .push(Node2D::new(id, name).with_transform(transform));
        Ok(id)
    }

    pub fn restore_node(&mut self, node: Node2D) -> Result<EntityId, SceneError> {
        if node.name.is_empty() {
            return Err(SceneError::EmptyNodeName);
        }
        if !node.transform.is_finite() {
            return Err(SceneError::InvalidTransform);
        }
        if self.node(node.id).is_some() {
            return Err(SceneError::EntityAlreadyExists);
        }

        let id = node.id;
        let next_id = id
            .raw()
            .checked_add(1)
            .ok_or(SceneError::EntityIdExhausted)?;
        self.next_id = self.next_id.max(next_id);
        self.nodes.push(node);
        Ok(id)
    }

    pub fn nodes(&self) -> &[Node2D] {
        &self.nodes
    }

    pub fn find_node_by_name(&self, name: &str) -> Option<&Node2D> {
        self.nodes.iter().find(|node| node.name == name)
    }

    pub fn find_node_by_tag(&self, tag: &str) -> Option<&Node2D> {
        self.nodes.iter().find(|node| {
            self.components
                .tag(node.id)
                .map(|component| component.tag == tag)
                .unwrap_or(false)
        })
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

    pub fn remove_tag(&mut self, entity: EntityId) -> Result<Option<TagComponent>, SceneError> {
        self.ensure_entity_exists(entity)?;
        Ok(self.components.remove_tag(entity))
    }

    pub fn add_sprite(
        &mut self,
        entity: EntityId,
        component: SpriteComponent,
    ) -> Result<(), SceneError> {
        self.ensure_entity_exists(entity)?;
        if component.sprite_path.is_empty() {
            return Err(SceneError::EmptyAssetPath);
        }
        self.components.insert_sprite(entity, component);
        Ok(())
    }

    pub fn sprite(&self, entity: EntityId) -> Option<&SpriteComponent> {
        self.components.sprite(entity)
    }

    pub fn remove_sprite(
        &mut self,
        entity: EntityId,
    ) -> Result<Option<SpriteComponent>, SceneError> {
        self.ensure_entity_exists(entity)?;
        Ok(self.components.remove_sprite(entity))
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

    pub fn add_tilemap(
        &mut self,
        entity: EntityId,
        component: TilemapComponent,
    ) -> Result<(), SceneError> {
        self.ensure_entity_exists(entity)?;
        component.validate().map_err(SceneError::InvalidTilemap)?;
        self.components.insert_tilemap(entity, component);
        Ok(())
    }

    pub fn tilemap(&self, entity: EntityId) -> Option<&TilemapComponent> {
        self.components.tilemap(entity)
    }

    pub fn tilemap_mut(&mut self, entity: EntityId) -> Option<&mut TilemapComponent> {
        self.components.tilemap_mut(entity)
    }

    pub fn remove_tilemap(
        &mut self,
        entity: EntityId,
    ) -> Result<Option<TilemapComponent>, SceneError> {
        self.ensure_entity_exists(entity)?;
        Ok(self.components.remove_tilemap(entity))
    }

    pub fn tilemaps(&self) -> impl Iterator<Item = (EntityId, &TilemapComponent)> {
        self.components.tilemaps()
    }

    pub fn add_velocity(
        &mut self,
        entity: EntityId,
        component: Velocity2DComponent,
    ) -> Result<(), SceneError> {
        self.ensure_entity_exists(entity)?;
        if !component.is_finite() {
            return Err(SceneError::InvalidVelocity);
        }
        self.components.insert_velocity(entity, component);
        Ok(())
    }

    pub fn velocity(&self, entity: EntityId) -> Option<&Velocity2DComponent> {
        self.components.velocity(entity)
    }

    pub fn velocity_mut(&mut self, entity: EntityId) -> Option<&mut Velocity2DComponent> {
        self.components.velocity_mut(entity)
    }

    pub fn remove_velocity(
        &mut self,
        entity: EntityId,
    ) -> Result<Option<Velocity2DComponent>, SceneError> {
        self.ensure_entity_exists(entity)?;
        Ok(self.components.remove_velocity(entity))
    }

    pub fn velocities(&self) -> impl Iterator<Item = (EntityId, &Velocity2DComponent)> {
        self.components.velocities()
    }

    pub fn add_collider(
        &mut self,
        entity: EntityId,
        component: Collider2DComponent,
    ) -> Result<(), SceneError> {
        self.ensure_entity_exists(entity)?;
        if !component.is_valid() {
            return Err(SceneError::InvalidCollider);
        }
        self.components.insert_collider(entity, component);
        Ok(())
    }

    pub fn collider(&self, entity: EntityId) -> Option<&Collider2DComponent> {
        self.components.collider(entity)
    }

    pub fn remove_collider(
        &mut self,
        entity: EntityId,
    ) -> Result<Option<Collider2DComponent>, SceneError> {
        self.ensure_entity_exists(entity)?;
        Ok(self.components.remove_collider(entity))
    }

    pub fn colliders(&self) -> impl Iterator<Item = (EntityId, &Collider2DComponent)> {
        self.components.colliders()
    }

    pub fn node(&self, id: EntityId) -> Option<&Node2D> {
        self.nodes.iter().find(|node| node.id == id)
    }

    pub fn node_mut(&mut self, id: EntityId) -> Option<&mut Node2D> {
        self.nodes.iter_mut().find(|node| node.id == id)
    }

    pub fn despawn_node(&mut self, id: EntityId) -> Result<Node2D, SceneError> {
        let index = self
            .nodes
            .iter()
            .position(|node| node.id == id)
            .ok_or(SceneError::EntityNotFound)?;
        self.components.remove_all(id);
        Ok(self.nodes.remove(index))
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
    EntityAlreadyExists,
    EntityIdExhausted,
    EntityNotFound,
    EmptyAssetPath,
    EmptyNodeName,
    EmptyTag,
    InvalidCameraZoom,
    InvalidCollider,
    InvalidTilemap(TilemapError),
    InvalidTransform,
    InvalidVelocity,
}

impl fmt::Display for SceneError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EntityAlreadyExists => formatter.write_str("scene entity already exists"),
            Self::EntityIdExhausted => formatter.write_str("scene entity id space was exhausted"),
            Self::EntityNotFound => formatter.write_str("scene entity was not found"),
            Self::EmptyAssetPath => formatter.write_str("sprite asset path cannot be empty"),
            Self::EmptyNodeName => formatter.write_str("node name cannot be empty"),
            Self::EmptyTag => formatter.write_str("tag cannot be empty"),
            Self::InvalidCameraZoom => {
                formatter.write_str("camera zoom must be finite and positive")
            }
            Self::InvalidCollider => formatter.write_str(
                "collider half extents and offset must be finite, with positive half extents",
            ),
            Self::InvalidTilemap(error) => write!(formatter, "invalid tilemap: {error}"),
            Self::InvalidTransform => formatter.write_str("transform contains non-finite values"),
            Self::InvalidVelocity => formatter.write_str("velocity contains non-finite values"),
        }
    }
}

impl Error for SceneError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidTilemap(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Camera2DComponent, Collider2DComponent, EntityId, Node2D, Scene, SceneError,
        SpriteComponent, TagComponent, TileCell, TileSize, TilemapComponent, TilemapError,
        TilemapSize, Transform2D, Vec2, Velocity2DComponent,
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
        let world = scene.spawn_node("World");

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
        scene
            .add_tilemap(world, test_tilemap())
            .expect("tilemap should attach");
        scene
            .add_velocity(player, Velocity2DComponent::from_xy(80.0, 0.0))
            .expect("velocity should attach");
        scene
            .add_collider(
                player,
                Collider2DComponent::rectangle(Vec2::new(16.0, 24.0)),
            )
            .expect("collider should attach");

        assert_eq!(scene.tag(player).expect("tag exists").tag, "player");
        assert_eq!(
            scene.sprite(player).expect("sprite exists").sprite_path,
            "sprites/player.png"
        );
        assert_eq!(scene.camera(camera).expect("camera exists").zoom, 2.0);
        assert!(scene.tilemap(world).is_some());
        assert_eq!(
            scene.velocity(player).expect("velocity exists").linear,
            Vec2::new(80.0, 0.0)
        );
        assert!(scene.collider(player).is_some());
    }

    #[test]
    fn components_reject_missing_entities() {
        let mut scene = Scene::new("Test Scene");
        let missing = EntityId::from_raw(999);

        let result = scene.add_sprite(missing, SpriteComponent::new("sprites/missing.png"));

        assert_eq!(result, Err(SceneError::EntityNotFound));
    }

    #[test]
    fn tilemaps_can_be_mutated_for_painting() {
        let mut scene = Scene::new("Test Scene");
        let world = scene.spawn_node("World");
        scene
            .add_tilemap(world, test_tilemap())
            .expect("tilemap should attach");

        let previous = scene
            .tilemap_mut(world)
            .expect("tilemap exists")
            .set_tile("Ground", 1, 1, Some(TileCell::new(3)))
            .expect("tile should paint");

        assert_eq!(previous, None);
        assert_eq!(
            scene
                .tilemap(world)
                .expect("tilemap exists")
                .tile("Ground", 1, 1),
            Ok(Some(TileCell::new(3)))
        );
    }

    #[test]
    fn invalid_tilemaps_are_rejected() {
        let mut scene = Scene::new("Test Scene");
        let world = scene.spawn_node("World");
        let mut tilemap = test_tilemap();
        tilemap.layers.clear();

        let result = scene.add_tilemap(world, tilemap);

        assert_eq!(
            result,
            Err(SceneError::InvalidTilemap(TilemapError::MissingLayer))
        );
    }

    #[test]
    fn despawn_node_removes_node_and_attached_components() {
        let mut scene = Scene::new("Test Scene");
        let world = scene.spawn_node("World");
        scene
            .add_tag(world, TagComponent::new("world"))
            .expect("tag should attach");
        scene
            .add_tilemap(world, test_tilemap())
            .expect("tilemap should attach");

        let removed = scene.despawn_node(world).expect("node should despawn");

        assert_eq!(removed.name, "World");
        assert!(scene.node(world).is_none());
        assert!(scene.tag(world).is_none());
        assert!(scene.tilemap(world).is_none());
    }

    #[test]
    fn runtime_components_validate_data() {
        let mut scene = Scene::new("Test Scene");
        let player = scene.spawn_node("Player");

        let velocity = Velocity2DComponent::new(Vec2::new(f32::NAN, 0.0));
        let invalid_collider = Collider2DComponent::new(Vec2::ZERO);

        assert_eq!(
            scene.add_velocity(player, velocity),
            Err(SceneError::InvalidVelocity)
        );
        assert_eq!(
            scene.add_collider(player, invalid_collider),
            Err(SceneError::InvalidCollider)
        );
        assert!(scene.velocity(player).is_none());
        assert!(scene.collider(player).is_none());
    }

    #[test]
    fn restore_node_preserves_id_and_advances_next_id() {
        let mut scene = Scene::new("Test Scene");
        let node = Node2D::new(EntityId::from_raw(7), "Restored");

        let restored = scene.restore_node(node).expect("node should restore");
        let next = scene.spawn_node("Next");

        assert_eq!(restored.raw(), 7);
        assert_eq!(next.raw(), 8);
    }

    fn test_tilemap() -> TilemapComponent {
        TilemapComponent::new(TilemapSize::new(4, 4), TileSize::new(32, 32))
            .expect("tilemap should be valid")
    }
}
