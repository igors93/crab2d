use std::error::Error;
use std::fmt;

use crab2d_scene::{Aabb2D, EntityId, Scene};

#[derive(Debug, Clone, PartialEq)]
pub struct FrameStep {
    pub delta_seconds: f32,
    pub moved_entities: usize,
    pub collisions: Vec<CollisionEvent>,
}

impl FrameStep {
    pub fn empty(delta_seconds: f32) -> Self {
        Self {
            delta_seconds,
            moved_entities: 0,
            collisions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionEvent {
    pub a: EntityId,
    pub b: EntityId,
    pub aabb_a: Aabb2D,
    pub aabb_b: Aabb2D,
    pub includes_sensor: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineTickError {
    InvalidDelta,
}

impl fmt::Display for EngineTickError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDelta => {
                formatter.write_str("engine tick delta must be finite and non-negative")
            }
        }
    }
}

impl Error for EngineTickError {}

pub(crate) fn run_scene_systems(
    scene: &mut Scene,
    delta_seconds: f32,
) -> Result<FrameStep, EngineTickError> {
    if !delta_seconds.is_finite() || delta_seconds < 0.0 {
        return Err(EngineTickError::InvalidDelta);
    }

    let mut frame = FrameStep::empty(delta_seconds);
    let movements = scene
        .velocities()
        .map(|(entity, velocity)| (entity, velocity.linear * delta_seconds))
        .collect::<Vec<_>>();

    for (entity, delta) in movements {
        if delta.length_squared() == 0.0 {
            continue;
        }
        if let Some(node) = scene.node_mut(entity) {
            node.transform.position += delta;
            frame.moved_entities += 1;
        }
    }

    frame.collisions = detect_collisions(scene);
    Ok(frame)
}

fn detect_collisions(scene: &Scene) -> Vec<CollisionEvent> {
    let colliders = scene
        .colliders()
        .filter_map(|(entity, collider)| {
            scene.node(entity).map(|node| {
                (
                    entity,
                    collider.world_aabb(node.transform),
                    collider.is_sensor,
                )
            })
        })
        .collect::<Vec<_>>();

    let mut collisions = Vec::new();
    for (index, (a, aabb_a, sensor_a)) in colliders.iter().enumerate() {
        for (b, aabb_b, sensor_b) in colliders.iter().skip(index + 1) {
            if aabb_a.intersects(*aabb_b) {
                collisions.push(CollisionEvent {
                    a: *a,
                    b: *b,
                    aabb_a: *aabb_a,
                    aabb_b: *aabb_b,
                    includes_sensor: *sensor_a || *sensor_b,
                });
            }
        }
    }
    collisions
}

#[cfg(test)]
mod tests {
    use crab2d_scene::{Collider2DComponent, Scene, Transform2D, Vec2, Velocity2DComponent};

    use super::{run_scene_systems, EngineTickError};

    #[test]
    fn scene_systems_move_nodes_with_velocity() {
        let mut scene = Scene::new("Runtime Test");
        let player = scene
            .spawn_node_with_transform("Player", Transform2D::from_position(Vec2::new(2.0, 3.0)))
            .expect("node should spawn");
        scene
            .add_velocity(player, Velocity2DComponent::from_xy(10.0, -2.0))
            .expect("velocity should attach");

        let frame = run_scene_systems(&mut scene, 0.5).expect("tick should succeed");

        assert_eq!(frame.moved_entities, 1);
        assert_eq!(
            scene
                .node(player)
                .expect("player exists")
                .transform
                .position,
            Vec2::new(7.0, 2.0)
        );
    }

    #[test]
    fn scene_systems_report_aabb_collisions() {
        let mut scene = Scene::new("Runtime Test");
        let player = scene
            .spawn_node_with_transform("Player", Transform2D::from_position(Vec2::new(0.0, 0.0)))
            .expect("node should spawn");
        let wall = scene
            .spawn_node_with_transform("Wall", Transform2D::from_position(Vec2::new(12.0, 0.0)))
            .expect("node should spawn");
        scene
            .add_collider(
                player,
                Collider2DComponent::rectangle(Vec2::new(16.0, 16.0)),
            )
            .expect("collider should attach");
        scene
            .add_collider(
                wall,
                Collider2DComponent::rectangle(Vec2::new(16.0, 16.0)).sensor(),
            )
            .expect("collider should attach");

        let frame = run_scene_systems(&mut scene, 0.0).expect("tick should succeed");

        assert_eq!(frame.collisions.len(), 1);
        assert_eq!(frame.collisions[0].a, player);
        assert_eq!(frame.collisions[0].b, wall);
        assert!(frame.collisions[0].includes_sensor);
    }

    #[test]
    fn scene_systems_reject_invalid_delta() {
        let mut scene = Scene::new("Runtime Test");

        let result = run_scene_systems(&mut scene, f32::NAN);

        assert_eq!(result, Err(EngineTickError::InvalidDelta));
    }
}
