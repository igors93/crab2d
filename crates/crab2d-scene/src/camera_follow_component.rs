use serde::{Deserialize, Serialize};

use crate::{Aabb2D, EntityId, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CameraFollowComponent {
    pub target: EntityId,
    pub smoothing: f32,
    pub enabled: bool,
    /// When true, the camera X position is not updated (locks horizontal axis).
    #[serde(default)]
    pub lock_x: bool,
    /// When true, the camera Y position is not updated (locks vertical axis).
    #[serde(default)]
    pub lock_y: bool,
    /// Radius around the camera center where target movement does not move the camera.
    /// Set to 0.0 to disable the dead zone.
    #[serde(default)]
    pub dead_zone: f32,
    /// Optional world-space bounds that clamp the camera position.
    #[serde(default)]
    pub bounds: Option<Aabb2D>,
}

impl CameraFollowComponent {
    pub const fn new(target: EntityId) -> Self {
        Self {
            target,
            smoothing: 0.0,
            enabled: true,
            lock_x: false,
            lock_y: false,
            dead_zone: 0.0,
            bounds: None,
        }
    }

    pub const fn with_smoothing(mut self, smoothing: f32) -> Self {
        self.smoothing = smoothing;
        self
    }

    pub const fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    pub const fn with_lock_x(mut self) -> Self {
        self.lock_x = true;
        self
    }

    pub const fn with_lock_y(mut self) -> Self {
        self.lock_y = true;
        self
    }

    pub const fn with_dead_zone(mut self, radius: f32) -> Self {
        self.dead_zone = radius;
        self
    }

    pub fn with_bounds(mut self, bounds: Aabb2D) -> Self {
        self.bounds = Some(bounds);
        self
    }

    pub fn is_valid(self) -> bool {
        self.smoothing.is_finite()
            && self.smoothing >= 0.0
            && self.dead_zone.is_finite()
            && self.dead_zone >= 0.0
    }
}

/// Compute the desired camera offset toward `target` respecting dead_zone,
/// lock_x/lock_y and optional world bounds. Returns the new camera position.
pub fn compute_camera_position(
    camera_pos: Vec2,
    target_pos: Vec2,
    follow: CameraFollowComponent,
    factor: f32,
) -> Vec2 {
    let mut delta = target_pos - camera_pos;

    // Dead zone: don't move if target is inside the radius
    if follow.dead_zone > 0.0 {
        let dist = delta.length();
        if dist <= follow.dead_zone {
            return camera_pos;
        }
        // Move only the portion outside the dead zone
        let direction = delta * (1.0 / dist);
        delta = direction * (dist - follow.dead_zone);
    }

    let mut new_pos = camera_pos;
    if !follow.lock_x {
        new_pos.x += delta.x * factor;
    }
    if !follow.lock_y {
        new_pos.y += delta.y * factor;
    }

    // Clamp to world bounds
    if let Some(bounds) = follow.bounds {
        new_pos.x = new_pos.x.clamp(bounds.min.x, bounds.max.x);
        new_pos.y = new_pos.y.clamp(bounds.min.y, bounds.max.y);
    }

    new_pos
}
