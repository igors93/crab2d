use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameFlow {
    pub initial_scene: String,
    pub scenes: Vec<String>,
    pub title_scene: Option<String>,
    pub pause_scene: Option<String>,
    pub game_over_scene: Option<String>,
    pub victory_scene: Option<String>,
    pub game_title: String,
    pub target_fps: u32,
}

impl Default for GameFlow {
    fn default() -> Self {
        Self {
            initial_scene: "Main Scene".to_string(),
            scenes: vec!["Main Scene".to_string()],
            title_scene: None,
            pause_scene: None,
            game_over_scene: None,
            victory_scene: None,
            game_title: "My Game".to_string(),
            target_fps: 60,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_flow_default_values_are_sensible() {
        let flow = GameFlow::default();
        assert_eq!(flow.initial_scene, "Main Scene");
        assert_eq!(flow.game_title, "My Game");
        assert_eq!(flow.target_fps, 60);
        assert!(flow.title_scene.is_none());
        assert!(flow.pause_scene.is_none());
    }
}
