use crate::{EntityId, Transform2D};

#[derive(Debug, Clone)]
pub struct Node2D {
    pub id: EntityId,
    pub name: String,
    pub transform: Transform2D,
}

impl Node2D {
    pub fn new(id: EntityId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            transform: Transform2D::default(),
        }
    }

    pub fn with_transform(mut self, transform: Transform2D) -> Self {
        self.transform = transform;
        self
    }
}
