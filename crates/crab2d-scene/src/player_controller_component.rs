use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PlayerControllerComponent {
    pub move_speed: f32,
    pub enabled: bool,
}

impl PlayerControllerComponent {
    pub const DEFAULT_MOVE_SPEED: f32 = 160.0;

    pub const fn new(move_speed: f32) -> Self {
        Self {
            move_speed,
            enabled: true,
        }
    }

    pub const fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    pub fn is_valid(self) -> bool {
        self.move_speed.is_finite() && self.move_speed >= 0.0
    }
}

impl Default for PlayerControllerComponent {
    fn default() -> Self {
        Self::new(Self::DEFAULT_MOVE_SPEED)
    }
}

#[cfg(test)]
mod tests {
    use super::PlayerControllerComponent;

    #[test]
    fn player_controller_rejects_invalid_speed() {
        assert!(!PlayerControllerComponent::new(f32::NAN).is_valid());
        assert!(!PlayerControllerComponent::new(-1.0).is_valid());
        assert!(PlayerControllerComponent::new(0.0).is_valid());
        assert!(PlayerControllerComponent::new(120.0).is_valid());
    }
}
