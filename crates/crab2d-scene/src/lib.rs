#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId(u64);

impl EntityId {
    pub fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform2D {
    pub position: Vec2,
    pub rotation_radians: f32,
    pub scale: Vec2,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation_radians: 0.0,
            scale: Vec2::ONE,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Node2D {
    pub id: EntityId,
    pub name: String,
    pub transform: Transform2D,
}

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
        let id = EntityId(self.next_id);
        self.next_id += 1;
        self.nodes.push(Node2D {
            id,
            name: name.into(),
            transform: Transform2D::default(),
        });
        id
    }

    pub fn nodes(&self) -> &[Node2D] {
        &self.nodes
    }
}
