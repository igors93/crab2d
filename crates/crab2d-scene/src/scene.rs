use std::error::Error;
use std::fmt;

use crate::{EntityId, Node2D, Transform2D};

#[derive(Debug, Clone)]
pub struct Scene {
    pub name: String,
    next_id: u64,
    nodes: Vec<Node2D>,
}

impl Scene {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            next_id: 0,
            nodes: Vec::new(),
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneError {
    EntityIdExhausted,
    InvalidTransform,
}

impl fmt::Display for SceneError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EntityIdExhausted => formatter.write_str("scene entity id space was exhausted"),
            Self::InvalidTransform => formatter.write_str("transform contains non-finite values"),
        }
    }
}

impl Error for SceneError {}

#[cfg(test)]
mod tests {
    use crate::{Scene, Transform2D, Vec2};

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
}
