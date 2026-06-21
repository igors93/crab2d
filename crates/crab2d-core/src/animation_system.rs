use crab2d_scene::{EntityId, Scene};

/// Advances all animation states and returns events for finished one-shot animations.
pub fn tick_animations(scene: &mut Scene, delta_seconds: f32) -> Vec<(EntityId, String)> {
    let entity_ids: Vec<_> = scene.animations().map(|(id, _)| id).collect();
    let mut ended = Vec::new();
    for entity_id in entity_ids {
        let Some(anim) = scene.animation_mut(entity_id) else {
            continue;
        };
        if !anim.playing {
            continue;
        }
        let Some(state) = anim
            .states
            .iter()
            .find(|s| s.name == anim.current_state)
            .cloned()
        else {
            continue;
        };
        if state.fps <= 0.0 || state.frames.is_empty() {
            continue;
        }
        anim.frame_timer += delta_seconds;
        let frame_duration = 1.0 / state.fps;
        while anim.frame_timer >= frame_duration {
            anim.frame_timer -= frame_duration;
            let next = anim.current_frame + 1;
            if next >= state.frames.len() as u32 {
                if state.looping {
                    anim.current_frame = 0;
                } else {
                    anim.playing = false;
                    ended.push((entity_id, state.name.clone()));
                }
            } else {
                anim.current_frame = next;
            }
        }
    }
    ended
}
