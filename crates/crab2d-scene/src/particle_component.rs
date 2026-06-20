use crate::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParticleEmitterComponent {
    pub texture_path: String,
    pub emit_rate: f32, // particles per second
    pub particle_lifetime: f32,
    pub speed_min: f32,
    pub speed_max: f32,
    pub direction: Vec2,     // normalized base direction
    pub spread_degrees: f32, // cone half-angle in degrees
    pub gravity_scale: f32,
    pub size_start: f32,
    pub size_end: f32,
    pub color_start: [u8; 4],
    pub color_end: [u8; 4],
    pub enabled: bool,
    pub max_particles: u32,
}

impl ParticleEmitterComponent {
    pub fn new(texture_path: impl Into<String>) -> Self {
        Self {
            texture_path: texture_path.into(),
            emit_rate: 20.0,
            particle_lifetime: 1.0,
            speed_min: 50.0,
            speed_max: 150.0,
            direction: Vec2::new(0.0, 1.0),
            spread_degrees: 30.0,
            gravity_scale: 0.5,
            size_start: 8.0,
            size_end: 0.0,
            color_start: [255, 255, 200, 255],
            color_end: [255, 100, 50, 0],
            enabled: true,
            max_particles: 256,
        }
    }
}

/// Runtime-only particle state (NOT serialized)
#[derive(Debug, Clone, Default)]
pub struct ParticleState {
    pub particles: Vec<Particle>,
    pub emit_accumulator: f32,
}

#[derive(Debug, Clone)]
pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub size: f32,
}

impl Particle {
    pub fn progress(&self) -> f32 {
        if self.max_lifetime <= 0.0 {
            return 1.0;
        }
        1.0 - (self.lifetime / self.max_lifetime)
    }
}
