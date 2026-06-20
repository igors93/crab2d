use serde::{Deserialize, Serialize};

use crate::{Transform2D, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Collider2DComponent {
    pub half_extents: Vec2,
    pub offset: Vec2,
    pub is_sensor: bool,
    pub collision_layer: u8, // bitmask: which layer this entity is on
    pub collision_mask: u8,  // bitmask: which layers this entity collides with
    pub one_way: bool,       // true = entity can pass through from below
    pub gravity_scale: f32,  // multiplier for gravity (0.0 = no gravity)
}

impl Collider2DComponent {
    pub const fn new(half_extents: Vec2) -> Self {
        Self {
            half_extents,
            offset: Vec2::ZERO,
            is_sensor: false,
            collision_layer: 1,
            collision_mask: 0xFF,
            one_way: false,
            gravity_scale: 0.0,
        }
    }

    pub fn rectangle(size: Vec2) -> Self {
        Self {
            half_extents: size * 0.5,
            is_sensor: false,
            offset: Vec2::ZERO,
            collision_layer: 1,
            collision_mask: 0xFF,
            one_way: false,
            gravity_scale: 0.0,
        }
    }

    pub const fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }

    pub const fn sensor(mut self) -> Self {
        self.is_sensor = true;
        self
    }

    pub const fn with_collision_layer(mut self, layer: u8) -> Self {
        self.collision_layer = layer;
        self
    }

    pub const fn with_collision_mask(mut self, mask: u8) -> Self {
        self.collision_mask = mask;
        self
    }

    pub const fn one_way(mut self) -> Self {
        self.one_way = true;
        self
    }

    pub const fn with_gravity_scale(mut self, scale: f32) -> Self {
        self.gravity_scale = scale;
        self
    }

    pub fn is_valid(self) -> bool {
        self.half_extents.is_finite()
            && self.offset.is_finite()
            && self.half_extents.x > 0.0
            && self.half_extents.y > 0.0
    }

    pub fn world_aabb(self, transform: Transform2D) -> Aabb2D {
        let scaled_half_extents = Vec2::new(
            self.half_extents.x * transform.scale.x.abs().max(0.0001),
            self.half_extents.y * transform.scale.y.abs().max(0.0001),
        );
        let center = transform.position + self.offset;
        Aabb2D::from_center_half_extents(center, scaled_half_extents)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Aabb2D {
    pub min: Vec2,
    pub max: Vec2,
}

impl Aabb2D {
    pub const fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    pub fn from_center_half_extents(center: Vec2, half_extents: Vec2) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    pub fn intersects(self, other: Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    pub fn center(self) -> Vec2 {
        (self.min + self.max) * 0.5
    }

    pub fn size(self) -> Vec2 {
        self.max - self.min
    }
}
