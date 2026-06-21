use crab2d_core::Engine;
use crab2d_scene::{
    CameraFollowComponent, Collider2DComponent, PlayerControllerComponent, Vec2,
    Velocity2DComponent,
};

pub(crate) fn ensure_runtime_defaults(engine: &mut Engine) {
    let player = engine
        .active_scene
        .find_node_by_tag("player")
        .or_else(|| engine.active_scene.find_node_by_name("Player"))
        .map(|node| node.id);

    if let Some(player) = player {
        if engine.active_scene.velocity(player).is_none() {
            let _ = engine
                .active_scene
                .add_velocity(player, Velocity2DComponent::default());
        }
        if engine.active_scene.player_controller(player).is_none() {
            let _ = engine
                .active_scene
                .add_player_controller(player, PlayerControllerComponent::default());
        }
        if engine.active_scene.collider(player).is_none() {
            let _ = engine.active_scene.add_collider(
                player,
                Collider2DComponent::rectangle(Vec2::new(24.0, 24.0)),
            );
        }

        if let Some(camera) = engine
            .active_scene
            .nodes()
            .iter()
            .find(|node| engine.active_scene.camera(node.id).is_some())
            .map(|node| node.id)
        {
            if engine.active_scene.camera_follow(camera).is_none() {
                let _ = engine
                    .active_scene
                    .add_camera_follow(camera, CameraFollowComponent::new(player));
            }
        }
    }
}
