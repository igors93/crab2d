//! Integration tests for the extended CameraFollowComponent (lock_x, lock_y, dead_zone, bounds).

#[cfg(test)]
mod tests {
    use crab2d_core::{Engine, EngineConfig};
    use crab2d_platform::InputState;
    use crab2d_scene::{
        Aabb2D, Camera2DComponent, CameraFollowComponent, Transform2D, Vec2, Velocity2DComponent,
    };

    fn make_engine_with_camera_following_player(
        follow: CameraFollowComponent,
    ) -> (Engine, crab2d_scene::EntityId, crab2d_scene::EntityId) {
        let mut engine = Engine::new(EngineConfig::new("Camera Follow Test"));
        let scene = &mut engine.active_scene;

        let player = scene
            .spawn_node_with_transform("Player", Transform2D::from_position(Vec2::new(100.0, 50.0)))
            .expect("player spawns");
        scene
            .add_velocity(player, Velocity2DComponent::from_xy(0.0, 0.0))
            .expect("velocity attaches");

        let camera = scene.spawn_node("Camera2D");
        scene
            .add_camera(camera, Camera2DComponent::default())
            .expect("camera attaches");
        scene
            .add_camera_follow(camera, follow)
            .expect("follow attaches");

        (engine, player, camera)
    }

    #[test]
    fn camera_snaps_to_target_with_no_smoothing() {
        let follow = CameraFollowComponent::new(crab2d_scene::EntityId::from_raw(0));
        let (mut engine, player, camera) = make_engine_with_camera_following_player(
            CameraFollowComponent::new(crab2d_scene::EntityId::from_raw(0)),
        );

        let _ = follow;
        let player_id = player;
        let follow = CameraFollowComponent::new(player_id);
        engine
            .active_scene
            .add_camera_follow(camera, follow)
            .expect("follow attaches");

        engine
            .tick_with_input(1.0 / 60.0, &InputState::default())
            .expect("tick succeeds");

        let camera_pos = engine
            .active_scene
            .node(camera)
            .expect("camera exists")
            .transform
            .position;
        let player_pos = engine
            .active_scene
            .node(player)
            .expect("player exists")
            .transform
            .position;

        assert_eq!(
            camera_pos, player_pos,
            "camera should snap exactly to player with no smoothing"
        );
    }

    #[test]
    fn lock_x_prevents_horizontal_tracking() {
        let mut engine = Engine::new(EngineConfig::new("Lock X Test"));
        let player = engine
            .active_scene
            .spawn_node_with_transform("Player", Transform2D::from_position(Vec2::new(200.0, 50.0)))
            .expect("player spawns");
        let camera = engine.active_scene.spawn_node("Camera2D");
        engine
            .active_scene
            .add_camera(camera, Camera2DComponent::default())
            .expect("camera attaches");

        let follow = CameraFollowComponent::new(player).with_lock_x();
        engine
            .active_scene
            .add_camera_follow(camera, follow)
            .expect("follow attaches");

        engine
            .tick_with_input(1.0 / 60.0, &InputState::default())
            .expect("tick succeeds");

        let camera_pos = engine
            .active_scene
            .node(camera)
            .expect("camera exists")
            .transform
            .position;

        assert_eq!(
            camera_pos.x, 0.0,
            "lock_x should prevent camera from moving horizontally (x stayed at 0.0)"
        );
        assert_eq!(
            camera_pos.y, 50.0,
            "camera should still track Y without lock_y"
        );
    }

    #[test]
    fn lock_y_prevents_vertical_tracking() {
        let mut engine = Engine::new(EngineConfig::new("Lock Y Test"));
        let player = engine
            .active_scene
            .spawn_node_with_transform(
                "Player",
                Transform2D::from_position(Vec2::new(100.0, 300.0)),
            )
            .expect("player spawns");
        let camera = engine.active_scene.spawn_node("Camera2D");
        engine
            .active_scene
            .add_camera(camera, Camera2DComponent::default())
            .expect("camera attaches");

        let follow = CameraFollowComponent::new(player).with_lock_y();
        engine
            .active_scene
            .add_camera_follow(camera, follow)
            .expect("follow attaches");

        engine
            .tick_with_input(1.0 / 60.0, &InputState::default())
            .expect("tick succeeds");

        let camera_pos = engine
            .active_scene
            .node(camera)
            .expect("camera exists")
            .transform
            .position;

        assert_eq!(
            camera_pos.y, 0.0,
            "lock_y should prevent camera from moving vertically (y stayed at 0.0)"
        );
        assert_eq!(
            camera_pos.x, 100.0,
            "camera should still track X without lock_x"
        );
    }

    #[test]
    fn dead_zone_prevents_movement_when_target_is_close() {
        let mut engine = Engine::new(EngineConfig::new("Dead Zone Test"));
        // Camera starts at (0,0), player is at (10,0) — inside a 50px dead zone.
        let player = engine
            .active_scene
            .spawn_node_with_transform("Player", Transform2D::from_position(Vec2::new(10.0, 0.0)))
            .expect("player spawns");
        let camera = engine.active_scene.spawn_node("Camera2D");
        engine
            .active_scene
            .add_camera(camera, Camera2DComponent::default())
            .expect("camera attaches");

        let follow = CameraFollowComponent::new(player).with_dead_zone(50.0);
        engine
            .active_scene
            .add_camera_follow(camera, follow)
            .expect("follow attaches");

        engine
            .tick_with_input(1.0 / 60.0, &InputState::default())
            .expect("tick succeeds");

        let camera_pos = engine
            .active_scene
            .node(camera)
            .expect("camera exists")
            .transform
            .position;

        assert_eq!(
            camera_pos,
            Vec2::ZERO,
            "camera should not move when target is inside dead zone"
        );
    }

    #[test]
    fn bounds_clamp_camera_position() {
        let mut engine = Engine::new(EngineConfig::new("Bounds Test"));
        // Camera starts at (0,0), player is far outside the clamped bounds.
        let player = engine
            .active_scene
            .spawn_node_with_transform(
                "Player",
                Transform2D::from_position(Vec2::new(9999.0, 9999.0)),
            )
            .expect("player spawns");
        let camera = engine.active_scene.spawn_node("Camera2D");
        engine
            .active_scene
            .add_camera(camera, Camera2DComponent::default())
            .expect("camera attaches");

        let bounds = Aabb2D {
            min: Vec2::new(-100.0, -100.0),
            max: Vec2::new(100.0, 100.0),
        };
        let follow = CameraFollowComponent::new(player).with_bounds(bounds);
        engine
            .active_scene
            .add_camera_follow(camera, follow)
            .expect("follow attaches");

        engine
            .tick_with_input(1.0 / 60.0, &InputState::default())
            .expect("tick succeeds");

        let camera_pos = engine
            .active_scene
            .node(camera)
            .expect("camera exists")
            .transform
            .position;

        assert!(
            camera_pos.x <= 100.0,
            "camera X should be clamped to bounds.max.x (got {})",
            camera_pos.x
        );
        assert!(
            camera_pos.y <= 100.0,
            "camera Y should be clamped to bounds.max.y (got {})",
            camera_pos.y
        );
    }
}
