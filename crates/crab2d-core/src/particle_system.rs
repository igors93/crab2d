use crab2d_scene::{EntityId, Particle, ParticleEmitterComponent, ParticleState, Scene, Vec2};
use rand::Rng;
use std::collections::HashMap;

pub struct ParticleSystem {
    pub states: HashMap<EntityId, ParticleState>,
}

impl ParticleSystem {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    pub fn tick(&mut self, scene: &Scene, delta_seconds: f32) {
        let emitters: Vec<(EntityId, ParticleEmitterComponent)> = scene
            .particle_emitters()
            .map(|(id, e)| (id, e.clone()))
            .collect();

        let mut rng = rand::thread_rng();

        for (entity_id, emitter) in &emitters {
            if !emitter.enabled {
                continue;
            }
            let Some(node) = scene.node(*entity_id) else {
                continue;
            };
            let origin = node.transform.position;

            let state = self.states.entry(*entity_id).or_default();

            // Age existing particles
            state.particles.retain_mut(|p| {
                p.lifetime -= delta_seconds;
                p.velocity.y -= 980.0 * emitter.gravity_scale * delta_seconds;
                p.position += p.velocity * delta_seconds;
                p.lifetime > 0.0
            });

            // Spawn new particles
            state.emit_accumulator += emitter.emit_rate * delta_seconds;
            let to_spawn = state.emit_accumulator as u32;
            state.emit_accumulator -= to_spawn as f32;

            for _ in 0..to_spawn {
                if state.particles.len() >= emitter.max_particles as usize {
                    break;
                }
                let angle_offset =
                    (rng.gen::<f32>() - 0.5) * 2.0 * emitter.spread_degrees.to_radians();
                let base_angle = emitter.direction.y.atan2(emitter.direction.x);
                let final_angle = base_angle + angle_offset;
                let speed =
                    rng.gen_range(emitter.speed_min..=emitter.speed_max.max(emitter.speed_min));
                let velocity = Vec2::new(final_angle.cos() * speed, final_angle.sin() * speed);
                state.particles.push(Particle {
                    position: origin,
                    velocity,
                    lifetime: emitter.particle_lifetime,
                    max_lifetime: emitter.particle_lifetime,
                    size: emitter.size_start,
                });
            }
        }

        // Clean up states for removed emitters
        let active_ids: std::collections::HashSet<EntityId> =
            emitters.iter().map(|(id, _)| *id).collect();
        self.states.retain(|id, _| active_ids.contains(id));
    }

    pub fn get_state(&self, entity_id: EntityId) -> Option<&ParticleState> {
        self.states.get(&entity_id)
    }
}

impl Default for ParticleSystem {
    fn default() -> Self {
        Self::new()
    }
}
