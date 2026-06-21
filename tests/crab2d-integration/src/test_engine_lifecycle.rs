//! End-to-end tests covering a multi-system frame: physics, collision, triggers.

#[cfg(test)]
mod tests {
    use crab2d_core::{Engine, EngineConfig};
    use crab2d_platform::{InputState, KeyCode, PlatformEvent};
    use crab2d_scene::{
        Collider2DComponent, PlayerControllerComponent, TagComponent, Transform2D,
        TriggerComponent, Vec2, Velocity2DComponent,
    };

    fn make_engine() -> Engine {
        Engine::new(EngineConfig::new("Integration Test"))
    }

    #[test]
    fn player_moves_collides_and_fires_trigger_in_single_tick() {
        let mut engine = make_engine();
        let scene = &mut engine.active_scene;

        let player = scene
            .spawn_node_with_transform("Player", Transform2D::from_position(Vec2::new(0.0, 0.0)))
            .expect("player spawns");
        scene
            .add_tag(player, TagComponent::new("player"))
            .expect("tag attaches");
        scene
            .add_velocity(player, Velocity2DComponent::from_xy(100.0, 0.0))
            .expect("velocity attaches");
        scene
            .add_collider(
                player,
                Collider2DComponent::rectangle(Vec2::new(16.0, 16.0)),
            )
            .expect("collider attaches");

        // Player ends at x=100 (velocity=100, dt=1). Player AABB: 92..108.
        // Coin at x=96 → AABB 88..104 — guaranteed overlap at frame end.
        let coin = scene
            .spawn_node_with_transform("Coin", Transform2D::from_position(Vec2::new(96.0, 0.0)))
            .expect("coin spawns");
        scene
            .add_collider(
                coin,
                Collider2DComponent::rectangle(Vec2::new(16.0, 16.0)).sensor(),
            )
            .expect("coin collider attaches");
        scene
            .add_trigger(coin, TriggerComponent::new("collect_coin").once())
            .expect("trigger attaches");

        let frame = engine.tick(1.0).expect("tick succeeds");

        let player_pos = engine
            .active_scene
            .node(player)
            .expect("player exists")
            .transform
            .position;

        assert!(
            player_pos.x > 0.0,
            "player should have moved right (was {player_pos:?})"
        );
        assert!(
            frame
                .collisions
                .iter()
                .any(|c| { (c.a == player && c.b == coin) || (c.a == coin && c.b == player) }),
            "expected collision between player and coin"
        );
        assert!(
            frame.triggers.iter().any(|t| t.name == "collect_coin"),
            "expected collect_coin trigger to fire"
        );
    }

    #[test]
    fn input_drives_player_controller_and_camera_follow_in_same_tick() {
        use crab2d_scene::{Camera2DComponent, CameraFollowComponent};

        let mut engine = make_engine();
        let scene = &mut engine.active_scene;

        let player = scene.spawn_node("Player");
        scene
            .add_velocity(player, Velocity2DComponent::default())
            .expect("velocity attaches");
        scene
            .add_player_controller(player, PlayerControllerComponent::new(200.0))
            .expect("controller attaches");

        let camera = scene.spawn_node("Camera2D");
        scene
            .add_camera(camera, Camera2DComponent::default())
            .expect("camera attaches");
        scene
            .add_camera_follow(camera, CameraFollowComponent::new(player))
            .expect("follow attaches");

        let mut input = InputState::default();
        input.apply_event(PlatformEvent::KeyPressed(KeyCode::Character('d')));

        let frame = engine.tick_with_input(1.0, &input).expect("tick succeeds");

        let player_x = engine
            .active_scene
            .node(player)
            .expect("player exists")
            .transform
            .position
            .x;
        let camera_x = engine
            .active_scene
            .node(camera)
            .expect("camera exists")
            .transform
            .position
            .x;

        assert!(player_x > 0.0, "player should move right");
        assert_eq!(
            camera_x, player_x,
            "camera should snap to player (no smoothing)"
        );
        assert_eq!(frame.camera_updates, 1);
    }

    #[test]
    fn gravity_accelerates_entity_over_multiple_ticks() {
        let mut engine = make_engine();
        engine.active_scene.physics_settings_mut().enabled = true;

        let body = engine
            .active_scene
            .spawn_node_with_transform("Body", Transform2D::from_position(Vec2::new(0.0, 100.0)))
            .expect("body spawns");
        engine
            .active_scene
            .add_velocity(body, Velocity2DComponent::default())
            .expect("velocity attaches");
        engine
            .active_scene
            .add_collider(
                body,
                Collider2DComponent::rectangle(Vec2::new(8.0, 8.0)).with_gravity_scale(1.0),
            )
            .expect("collider attaches");

        for _ in 0..10 {
            engine.tick(1.0 / 60.0).expect("tick succeeds");
        }

        let y = engine
            .active_scene
            .node(body)
            .expect("body exists")
            .transform
            .position
            .y;

        assert!(y < 100.0, "body should fall (y < 100.0, got {y})");
    }
}
