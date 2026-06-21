use std::error::Error;
use std::fmt;

use crab2d_platform::{InputState, KeyCode};
use crab2d_scene::{compute_camera_position, Aabb2D, Collider2DComponent, EntityId, Scene, Vec2};

#[derive(Debug, Clone, PartialEq)]
pub struct FrameStep {
    pub delta_seconds: f32,
    pub moved_entities: usize,
    pub collisions: Vec<CollisionEvent>,
    pub solid_collisions: Vec<SolidCollisionEvent>,
    pub collision_resolutions: Vec<CollisionResolution>,
    pub triggers: Vec<TriggerEvent>,
    pub camera_updates: usize,
    /// Fired when a non-looping animation finishes: (entity, state_name)
    pub animation_ended: Vec<(EntityId, String)>,
}

impl FrameStep {
    pub fn empty(delta_seconds: f32) -> Self {
        Self {
            delta_seconds,
            moved_entities: 0,
            collisions: Vec::new(),
            solid_collisions: Vec::new(),
            collision_resolutions: Vec::new(),
            triggers: Vec::new(),
            camera_updates: 0,
            animation_ended: Vec::new(),
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
pub struct CollisionResolution {
    pub entity: EntityId,
    pub blocked_x: bool,
    pub blocked_y: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolidCollisionEvent {
    pub entity: EntityId,
    pub obstacle: SolidObstacle,
    pub axis: CollisionAxis,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SolidObstacle {
    Entity(EntityId),
    Tile {
        tilemap: EntityId,
        x: u32,
        y: u32,
        tile_index: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionAxis {
    X,
    Y,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerEvent {
    pub trigger_entity: EntityId,
    pub activator: EntityId,
    pub name: String,
    pub once: bool,
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
    input: &InputState,
    delta_seconds: f32,
) -> Result<FrameStep, EngineTickError> {
    if !delta_seconds.is_finite() || delta_seconds < 0.0 {
        return Err(EngineTickError::InvalidDelta);
    }

    apply_player_controllers(scene, input);

    // Apply gravity to entities that have gravity_scale > 0
    if scene.physics_settings().enabled {
        let gravity = scene.physics_settings().gravity;
        let terminal = scene.physics_settings().terminal_velocity;
        let collider_entities: Vec<_> = scene
            .colliders()
            .map(|(id, c)| (id, c.gravity_scale))
            .collect();
        for (entity, scale) in collider_entities {
            if scale == 0.0 {
                continue;
            }
            let Some(vel) = scene.velocity_mut(entity) else {
                continue;
            };
            vel.linear.y += gravity.y * scale * delta_seconds;
            vel.linear.y = vel.linear.y.clamp(-terminal, terminal);
        }
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

        let Some(collider) = scene.collider(entity).copied() else {
            if let Some(node) = scene.node_mut(entity) {
                node.transform.position += delta;
                frame.moved_entities += 1;
            }
            continue;
        };

        let obstacles = solid_obstacles(scene, entity);
        let blocked_x = move_axis(
            scene,
            entity,
            collider,
            Vec2::new(delta.x, 0.0),
            CollisionAxis::X,
            &obstacles,
            &mut frame.solid_collisions,
        );
        let blocked_y = move_axis(
            scene,
            entity,
            collider,
            Vec2::new(0.0, delta.y),
            CollisionAxis::Y,
            &obstacles,
            &mut frame.solid_collisions,
        );

        if let Some(node) = scene.node(entity) {
            if delta.length_squared() > 0.0 && node.transform.position.is_finite() {
                frame.moved_entities += 1;
            }
        }

        if blocked_x || blocked_y {
            frame.collision_resolutions.push(CollisionResolution {
                entity,
                blocked_x,
                blocked_y,
            });
        }
    }

    let (collisions, triggers) = detect_collisions_and_triggers(scene);
    frame.collisions = collisions;
    frame.triggers = triggers;
    frame.camera_updates = apply_camera_follow(scene, delta_seconds);

    Ok(frame)
}

fn apply_player_controllers(scene: &mut Scene, input: &InputState) {
    let direction = input_direction(input);
    let controllers = scene
        .player_controllers()
        .map(|(entity, controller)| (entity, *controller))
        .collect::<Vec<_>>();

    for (entity, controller) in controllers {
        let Some(velocity) = scene.velocity_mut(entity) else {
            continue;
        };
        velocity.linear = if controller.enabled {
            direction * controller.move_speed
        } else {
            Vec2::ZERO
        };
    }
}

fn input_direction(input: &InputState) -> Vec2 {
    let mut direction = Vec2::ZERO;

    // Keyboard
    if input.is_key_down(KeyCode::Character('a')) || input.is_key_down(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if input.is_key_down(KeyCode::Character('d')) || input.is_key_down(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }
    if input.is_key_down(KeyCode::Character('w')) || input.is_key_down(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if input.is_key_down(KeyCode::Character('s')) || input.is_key_down(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }

    // D-pad
    if input.is_key_down(KeyCode::Character('a')) {
    } else {
        use crab2d_platform::GamepadButton;
        if input.is_gamepad_down(GamepadButton::DpadLeft) {
            direction.x -= 1.0;
        }
        if input.is_gamepad_down(GamepadButton::DpadRight) {
            direction.x += 1.0;
        }
        if input.is_gamepad_down(GamepadButton::DpadUp) {
            direction.y += 1.0;
        }
        if input.is_gamepad_down(GamepadButton::DpadDown) {
            direction.y -= 1.0;
        }
    }

    // Left analog stick — only use if no keyboard direction already set
    if direction == Vec2::ZERO {
        let (sx, sy) = input.gamepad_left_stick();
        const DEAD_ZONE: f32 = 0.2;
        if sx.abs() > DEAD_ZONE || sy.abs() > DEAD_ZONE {
            direction = Vec2::new(sx, sy);
        }
    }

    direction.normalized_or_zero()
}

fn move_axis(
    scene: &mut Scene,
    entity: EntityId,
    collider: Collider2DComponent,
    delta: Vec2,
    axis: CollisionAxis,
    obstacles: &[SolidObstacleAabb],
    hits: &mut Vec<SolidCollisionEvent>,
) -> bool {
    if delta.length_squared() == 0.0 {
        return false;
    }

    let Some(node) = scene.node_mut(entity) else {
        return false;
    };
    let previous_aabb = collider.world_aabb(node.transform);
    node.transform.position += delta;

    let mut blocked = false;
    for obstacle in obstacles {
        let Some(node) = scene.node_mut(entity) else {
            break;
        };
        let moving_aabb = collider.world_aabb(node.transform);
        if !moving_aabb.intersects(obstacle.aabb) {
            continue;
        }
        if obstacle.one_way && !one_way_platform_blocks(previous_aabb, obstacle.aabb, delta, axis) {
            continue;
        }

        blocked = true;
        hits.push(SolidCollisionEvent {
            entity,
            obstacle: obstacle.target,
            axis,
        });

        match axis {
            CollisionAxis::X => {
                if delta.x > 0.0 {
                    node.transform.position.x -= moving_aabb.max.x - obstacle.aabb.min.x;
                } else if delta.x < 0.0 {
                    node.transform.position.x += obstacle.aabb.max.x - moving_aabb.min.x;
                }
            }
            CollisionAxis::Y => {
                if delta.y > 0.0 {
                    node.transform.position.y -= moving_aabb.max.y - obstacle.aabb.min.y;
                } else if delta.y < 0.0 {
                    node.transform.position.y += obstacle.aabb.max.y - moving_aabb.min.y;
                }
            }
        }
    }

    blocked
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SolidObstacleAabb {
    target: SolidObstacle,
    aabb: Aabb2D,
    one_way: bool,
}

fn one_way_platform_blocks(
    previous_aabb: Aabb2D,
    platform_aabb: Aabb2D,
    delta: Vec2,
    axis: CollisionAxis,
) -> bool {
    axis == CollisionAxis::Y && delta.y < 0.0 && previous_aabb.min.y >= platform_aabb.max.y
}

fn solid_obstacles(scene: &Scene, moving_entity: EntityId) -> Vec<SolidObstacleAabb> {
    let Some(moving_collider) = scene.collider(moving_entity).copied() else {
        return Vec::new();
    };
    let mut obstacles = scene
        .colliders()
        .filter_map(|(entity, collider)| {
            if entity == moving_entity || collider.is_sensor {
                return None;
            }
            if !collider_layers_match(moving_collider, *collider) {
                return None;
            }
            scene.node(entity).map(|node| SolidObstacleAabb {
                target: SolidObstacle::Entity(entity),
                aabb: collider.world_aabb(node.transform),
                one_way: collider.one_way,
            })
        })
        .collect::<Vec<_>>();

    for (tilemap_entity, tilemap) in scene.tilemaps() {
        let Some(node) = scene.node(tilemap_entity) else {
            continue;
        };
        let tile_width = tilemap.tile_size.width as f32;
        let tile_height = tilemap.tile_size.height as f32;
        for tile in tilemap.solid_tiles() {
            let center = Vec2::new(
                node.transform.position.x + tile.x as f32 * tile_width + tile_width / 2.0,
                node.transform.position.y + tile.y as f32 * tile_height + tile_height / 2.0,
            );
            obstacles.push(SolidObstacleAabb {
                target: SolidObstacle::Tile {
                    tilemap: tilemap_entity,
                    x: tile.x,
                    y: tile.y,
                    tile_index: tile.cell.tile_index,
                },
                aabb: Aabb2D::from_center_half_extents(
                    center,
                    Vec2::new(tile_width / 2.0, tile_height / 2.0),
                ),
                one_way: false,
            });
        }
    }

    obstacles
}

fn detect_collisions_and_triggers(scene: &Scene) -> (Vec<CollisionEvent>, Vec<TriggerEvent>) {
    let colliders = scene
        .colliders()
        .filter_map(|(entity, collider)| {
            scene.node(entity).map(|node| {
                (
                    entity,
                    *collider,
                    collider.world_aabb(node.transform),
                )
            })
        })
        .collect::<Vec<_>>();

    let mut collisions = Vec::new();
    let mut triggers = Vec::new();
    for (index, (a, collider_a, aabb_a)) in colliders.iter().enumerate() {
        for (b, collider_b, aabb_b) in colliders.iter().skip(index + 1) {
            if !collider_layers_match(*collider_a, *collider_b) {
                continue;
            }
            if !aabb_a.intersects(*aabb_b) {
                continue;
            }

            collisions.push(CollisionEvent {
                a: *a,
                b: *b,
                aabb_a: *aabb_a,
                aabb_b: *aabb_b,
                includes_sensor: collider_a.is_sensor || collider_b.is_sensor,
            });

            if collider_a.is_sensor {
                push_trigger(scene, &mut triggers, *a, *b);
            }
            if collider_b.is_sensor {
                push_trigger(scene, &mut triggers, *b, *a);
            }
        }
    }
    (collisions, triggers)
}

fn collider_layers_match(a: Collider2DComponent, b: Collider2DComponent) -> bool {
    a.collision_mask & b.collision_layer != 0 && b.collision_mask & a.collision_layer != 0
}

fn push_trigger(
    scene: &Scene,
    triggers: &mut Vec<TriggerEvent>,
    trigger_entity: EntityId,
    activator: EntityId,
) {
    let Some(trigger) = scene.trigger(trigger_entity) else {
        return;
    };
    triggers.push(TriggerEvent {
        trigger_entity,
        activator,
        name: trigger.name.clone(),
        once: trigger.once,
    });
}

fn apply_camera_follow(scene: &mut Scene, delta_seconds: f32) -> usize {
    let follows = scene
        .camera_follows()
        .map(|(entity, follow)| (entity, *follow))
        .collect::<Vec<_>>();
    let mut updates = 0;

    for (camera_entity, follow) in follows {
        if !follow.enabled {
            continue;
        }
        let Some(target_position) = scene
            .node(follow.target)
            .map(|node| node.transform.position)
        else {
            continue;
        };
        let Some(camera) = scene.node_mut(camera_entity) else {
            continue;
        };

        let factor = if follow.smoothing == 0.0 {
            1.0
        } else {
            (follow.smoothing * delta_seconds).clamp(0.0, 1.0)
        };
        camera.transform.position =
            compute_camera_position(camera.transform.position, target_position, follow, factor);
        updates += 1;
    }

    updates
}

#[cfg(test)]
mod tests {
    use crab2d_platform::{InputState, KeyCode, PlatformEvent};
    use crab2d_scene::{
        CameraFollowComponent, Collider2DComponent, PlayerControllerComponent, Scene, TileCell,
        TileSize, TilemapComponent, TilemapSize, Transform2D, TriggerComponent, Vec2,
        Velocity2DComponent,
    };

    use super::{
        input_direction, run_scene_systems, CollisionAxis, EngineTickError, SolidObstacle,
    };

    #[test]
    fn player_controller_input_sets_velocity() {
        let mut scene = Scene::new("Runtime Test");
        let player = scene.spawn_node("Player");
        scene
            .add_velocity(player, Velocity2DComponent::default())
            .expect("velocity should attach");
        scene
            .add_player_controller(player, PlayerControllerComponent::new(100.0))
            .expect("controller should attach");
        let input = input_with_keys([KeyCode::Character('d')]);

        run_scene_systems(&mut scene, &input, 0.0).expect("tick should succeed");

        assert_eq!(
            scene.velocity(player).expect("velocity exists").linear,
            Vec2::new(100.0, 0.0)
        );
    }

    #[test]
    fn diagonal_controller_input_is_normalized() {
        let input = input_with_keys([KeyCode::Character('d'), KeyCode::Character('w')]);

        let direction = input_direction(&input);

        assert!((direction.length() - 1.0).abs() < 0.001);
        assert!(direction.x > 0.0);
        assert!(direction.y > 0.0);
    }

    #[test]
    fn solid_collision_blocks_movement() {
        let mut scene = Scene::new("Runtime Test");
        let player = scene
            .spawn_node_with_transform("Player", Transform2D::from_position(Vec2::new(0.0, 0.0)))
            .expect("node should spawn");
        let wall = scene
            .spawn_node_with_transform("Wall", Transform2D::from_position(Vec2::new(16.0, 0.0)))
            .expect("node should spawn");
        scene
            .add_velocity(player, Velocity2DComponent::from_xy(32.0, 0.0))
            .expect("velocity should attach");
        scene
            .add_collider(
                player,
                Collider2DComponent::rectangle(Vec2::new(16.0, 16.0)),
            )
            .expect("collider should attach");
        scene
            .add_collider(wall, Collider2DComponent::rectangle(Vec2::new(16.0, 16.0)))
            .expect("collider should attach");

        let frame = run_scene_systems(&mut scene, &InputState::default(), 1.0)
            .expect("tick should succeed");

        assert_eq!(
            scene
                .node(player)
                .expect("player exists")
                .transform
                .position,
            Vec2::new(0.0, 0.0)
        );
        assert!(frame.collision_resolutions[0].blocked_x);
        assert_eq!(
            frame.solid_collisions[0].obstacle,
            SolidObstacle::Entity(wall)
        );
    }

    #[test]
    fn solid_collision_ignores_filtered_layers() {
        let mut scene = Scene::new("Runtime Test");
        let player = scene
            .spawn_node_with_transform("Player", Transform2D::from_position(Vec2::new(0.0, 0.0)))
            .expect("node should spawn");
        let wall = scene
            .spawn_node_with_transform("Wall", Transform2D::from_position(Vec2::new(16.0, 0.0)))
            .expect("node should spawn");
        scene
            .add_velocity(player, Velocity2DComponent::from_xy(16.0, 0.0))
            .expect("velocity should attach");
        scene
            .add_collider(
                player,
                Collider2DComponent::rectangle(Vec2::new(16.0, 16.0))
                    .with_collision_layer(0b0000_0001)
                    .with_collision_mask(0b0000_0100),
            )
            .expect("collider should attach");
        scene
            .add_collider(
                wall,
                Collider2DComponent::rectangle(Vec2::new(16.0, 16.0))
                    .with_collision_layer(0b0000_0010)
                    .with_collision_mask(0b0000_0001),
            )
            .expect("collider should attach");

        let frame = run_scene_systems(&mut scene, &InputState::default(), 1.0)
            .expect("tick should succeed");

        assert_eq!(
            scene
                .node(player)
                .expect("player exists")
                .transform
                .position,
            Vec2::new(16.0, 0.0)
        );
        assert!(frame.collisions.is_empty());
        assert!(frame.solid_collisions.is_empty());
        assert!(frame.collision_resolutions.is_empty());
    }

    #[test]
    fn sensor_generates_trigger_without_blocking() {
        let mut scene = Scene::new("Runtime Test");
        let player = scene
            .spawn_node_with_transform("Player", Transform2D::from_position(Vec2::new(0.0, 0.0)))
            .expect("node should spawn");
        let coin = scene
            .spawn_node_with_transform("Coin", Transform2D::from_position(Vec2::new(16.0, 0.0)))
            .expect("node should spawn");
        scene
            .add_velocity(player, Velocity2DComponent::from_xy(16.0, 0.0))
            .expect("velocity should attach");
        scene
            .add_collider(
                player,
                Collider2DComponent::rectangle(Vec2::new(16.0, 16.0)),
            )
            .expect("collider should attach");
        scene
            .add_collider(
                coin,
                Collider2DComponent::rectangle(Vec2::new(16.0, 16.0)).sensor(),
            )
            .expect("collider should attach");
        scene
            .add_trigger(coin, TriggerComponent::new("coin").once())
            .expect("trigger should attach");

        let frame = run_scene_systems(&mut scene, &InputState::default(), 1.0)
            .expect("tick should succeed");

        assert_eq!(
            scene
                .node(player)
                .expect("player exists")
                .transform
                .position,
            Vec2::new(16.0, 0.0)
        );
        assert_eq!(frame.triggers.len(), 1);
        assert_eq!(frame.triggers[0].name, "coin");
        assert!(frame.triggers[0].once);
    }

    #[test]
    fn sensor_trigger_respects_collision_layers() {
        let mut scene = Scene::new("Runtime Test");
        let player = scene
            .spawn_node_with_transform("Player", Transform2D::from_position(Vec2::new(0.0, 0.0)))
            .expect("node should spawn");
        let coin = scene
            .spawn_node_with_transform("Coin", Transform2D::from_position(Vec2::new(16.0, 0.0)))
            .expect("node should spawn");
        scene
            .add_velocity(player, Velocity2DComponent::from_xy(16.0, 0.0))
            .expect("velocity should attach");
        scene
            .add_collider(
                player,
                Collider2DComponent::rectangle(Vec2::new(16.0, 16.0))
                    .with_collision_layer(0b0000_0001)
                    .with_collision_mask(0b0000_0010),
            )
            .expect("collider should attach");
        scene
            .add_collider(
                coin,
                Collider2DComponent::rectangle(Vec2::new(16.0, 16.0))
                    .sensor()
                    .with_collision_layer(0b0000_0010)
                    .with_collision_mask(0b0000_0100),
            )
            .expect("collider should attach");
        scene
            .add_trigger(coin, TriggerComponent::new("coin").once())
            .expect("trigger should attach");

        let frame = run_scene_systems(&mut scene, &InputState::default(), 1.0)
            .expect("tick should succeed");

        assert!(frame.collisions.is_empty());
        assert!(frame.triggers.is_empty());
    }

    #[test]
    fn one_way_platform_blocks_downward_movement_from_above() {
        let mut scene = Scene::new("Runtime Test");
        let player = scene
            .spawn_node_with_transform("Player", Transform2D::from_position(Vec2::new(0.0, 24.0)))
            .expect("node should spawn");
        let platform = scene
            .spawn_node_with_transform(
                "One Way Platform",
                Transform2D::from_position(Vec2::new(0.0, 0.0)),
            )
            .expect("node should spawn");
        scene
            .add_velocity(player, Velocity2DComponent::from_xy(0.0, -24.0))
            .expect("velocity should attach");
        scene
            .add_collider(
                player,
                Collider2DComponent::rectangle(Vec2::new(16.0, 16.0)),
            )
            .expect("collider should attach");
        scene
            .add_collider(
                platform,
                Collider2DComponent::rectangle(Vec2::new(48.0, 16.0)).one_way(),
            )
            .expect("collider should attach");

        let frame = run_scene_systems(&mut scene, &InputState::default(), 1.0)
            .expect("tick should succeed");

        assert_eq!(
            scene
                .node(player)
                .expect("player exists")
                .transform
                .position,
            Vec2::new(0.0, 16.0)
        );
        assert!(frame.collision_resolutions[0].blocked_y);
        assert_eq!(
            frame.solid_collisions[0].obstacle,
            SolidObstacle::Entity(platform)
        );
        assert_eq!(frame.solid_collisions[0].axis, CollisionAxis::Y);
    }

    #[test]
    fn one_way_platform_allows_upward_movement_from_below() {
        let mut scene = Scene::new("Runtime Test");
        let player = scene
            .spawn_node_with_transform(
                "Player",
                Transform2D::from_position(Vec2::new(0.0, -24.0)),
            )
            .expect("node should spawn");
        let platform = scene
            .spawn_node_with_transform(
                "One Way Platform",
                Transform2D::from_position(Vec2::new(0.0, 0.0)),
            )
            .expect("node should spawn");
        scene
            .add_velocity(player, Velocity2DComponent::from_xy(0.0, 24.0))
            .expect("velocity should attach");
        scene
            .add_collider(
                player,
                Collider2DComponent::rectangle(Vec2::new(16.0, 16.0)),
            )
            .expect("collider should attach");
        scene
            .add_collider(
                platform,
                Collider2DComponent::rectangle(Vec2::new(48.0, 16.0)).one_way(),
            )
            .expect("collider should attach");

        let frame = run_scene_systems(&mut scene, &InputState::default(), 1.0)
            .expect("tick should succeed");

        assert_eq!(
            scene
                .node(player)
                .expect("player exists")
                .transform
                .position,
            Vec2::new(0.0, 0.0)
        );
        assert!(frame.solid_collisions.is_empty());
        assert!(frame.collision_resolutions.is_empty());
    }

    #[test]
    fn one_way_platform_allows_side_entry() {
        let mut scene = Scene::new("Runtime Test");
        let player = scene
            .spawn_node_with_transform("Player", Transform2D::from_position(Vec2::new(-32.0, 0.0)))
            .expect("node should spawn");
        let platform = scene
            .spawn_node_with_transform(
                "One Way Platform",
                Transform2D::from_position(Vec2::new(0.0, 0.0)),
            )
            .expect("node should spawn");
        scene
            .add_velocity(player, Velocity2DComponent::from_xy(32.0, 0.0))
            .expect("velocity should attach");
        scene
            .add_collider(
                player,
                Collider2DComponent::rectangle(Vec2::new(16.0, 16.0)),
            )
            .expect("collider should attach");
        scene
            .add_collider(
                platform,
                Collider2DComponent::rectangle(Vec2::new(48.0, 16.0)).one_way(),
            )
            .expect("collider should attach");

        let frame = run_scene_systems(&mut scene, &InputState::default(), 1.0)
            .expect("tick should succeed");

        assert_eq!(
            scene
                .node(player)
                .expect("player exists")
                .transform
                .position,
            Vec2::new(0.0, 0.0)
        );
        assert!(frame.solid_collisions.is_empty());
        assert!(frame.collision_resolutions.is_empty());
    }

    #[test]
    fn camera_follow_tracks_target() {
        let mut scene = Scene::new("Runtime Test");
        let player = scene
            .spawn_node_with_transform("Player", Transform2D::from_position(Vec2::new(24.0, 32.0)))
            .expect("node should spawn");
        let camera = scene.spawn_node("Camera2D");
        scene
            .add_camera_follow(camera, CameraFollowComponent::new(player))
            .expect("follow should attach");

        let frame = run_scene_systems(&mut scene, &InputState::default(), 1.0)
            .expect("tick should succeed");

        assert_eq!(frame.camera_updates, 1);
        assert_eq!(
            scene
                .node(camera)
                .expect("camera exists")
                .transform
                .position,
            Vec2::new(24.0, 32.0)
        );
    }

    #[test]
    fn solid_tile_blocks_movement() {
        let mut scene = Scene::new("Runtime Test");
        let player = scene
            .spawn_node_with_transform("Player", Transform2D::from_position(Vec2::new(0.0, 0.0)))
            .expect("node should spawn");
        let world = scene.spawn_node("World");
        let mut tilemap =
            TilemapComponent::new(TilemapSize::new(2, 1), TileSize::new(16, 16)).expect("tilemap");
        tilemap.collision.set_solid(3, true);
        tilemap
            .set_tile("Ground", 1, 0, Some(TileCell::new(3)))
            .expect("tile should set");
        scene
            .add_tilemap(world, tilemap)
            .expect("tilemap should attach");
        scene
            .add_velocity(player, Velocity2DComponent::from_xy(16.0, 0.0))
            .expect("velocity should attach");
        scene
            .add_collider(
                player,
                Collider2DComponent::rectangle(Vec2::new(16.0, 16.0)),
            )
            .expect("collider should attach");

        let frame = run_scene_systems(&mut scene, &InputState::default(), 1.0)
            .expect("tick should succeed");

        assert!(frame.collision_resolutions[0].blocked_x);
        assert_eq!(frame.solid_collisions[0].axis, CollisionAxis::X);
        assert!(matches!(
            frame.solid_collisions[0].obstacle,
            SolidObstacle::Tile { .. }
        ));
    }

    #[test]
    fn scene_systems_reject_invalid_delta() {
        let mut scene = Scene::new("Runtime Test");

        let result = run_scene_systems(&mut scene, &InputState::default(), f32::NAN);

        assert_eq!(result, Err(EngineTickError::InvalidDelta));
    }

    fn input_with_keys(keys: impl IntoIterator<Item = KeyCode>) -> InputState {
        let mut input = InputState::default();
        for key in keys {
            input.apply_event(PlatformEvent::KeyPressed(key));
        }
        input
    }
}
