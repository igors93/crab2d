use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;

use crab2d_assets::AssetRegistry;
use crab2d_scene::Scene;
use serde::{Deserialize, Serialize};

use crate::{Engine, ProjectInfo};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectDocument {
    pub project: ProjectInfo,
    pub assets: AssetRegistry,
    pub active_scene: Scene,
}

impl ProjectDocument {
    pub const DEFAULT_FILE_NAME: &'static str = "project.crab2d.json";

    pub fn new(project: ProjectInfo, assets: AssetRegistry, active_scene: Scene) -> Self {
        Self {
            project,
            assets,
            active_scene,
        }
    }

    pub fn from_engine(engine: &Engine) -> Self {
        Self::new(
            engine.project.clone(),
            engine.assets.clone(),
            engine.active_scene.clone(),
        )
    }

    pub fn apply_to_engine(self, engine: &mut Engine) {
        engine.project = self.project;
        engine.assets = self.assets;
        engine.active_scene = self.active_scene;
    }

    pub fn to_json_string(&self) -> Result<String, ProjectIoError> {
        serde_json::to_string_pretty(self).map_err(ProjectIoError::from)
    }

    pub fn from_json_str(input: &str) -> Result<Self, ProjectIoError> {
        serde_json::from_str(input).map_err(ProjectIoError::from)
    }

    pub fn save_to_path(&self, path: impl AsRef<Path>) -> Result<(), ProjectIoError> {
        let json = self.to_json_string()?;
        fs::write(path, json).map_err(ProjectIoError::from)
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, ProjectIoError> {
        let json = fs::read_to_string(path).map_err(ProjectIoError::from)?;
        Self::from_json_str(&json)
    }
}

#[derive(Debug)]
pub enum ProjectIoError {
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl fmt::Display for ProjectIoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "project file I/O failed: {error}"),
            Self::Json(error) => write!(formatter, "project JSON serialization failed: {error}"),
        }
    }
}

impl Error for ProjectIoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Json(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for ProjectIoError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for ProjectIoError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

#[cfg(test)]
mod tests {
    use crab2d_assets::AssetKind;
    use crab2d_scene::{
        Camera2DComponent, CameraFollowComponent, Collider2DComponent, PlayerControllerComponent,
        SpriteComponent, TagComponent, TriggerComponent, Vec2, Velocity2DComponent,
    };

    use super::*;
    use crate::EngineConfig;

    #[test]
    fn project_document_round_trips_through_json() {
        let mut engine = Engine::new(EngineConfig::new("Crab2D Test"));
        engine.open_project(ProjectInfo::new("Saved Project"));
        engine
            .assets
            .register(AssetKind::Sprite, "sprites/player.png");

        let player = engine.active_scene.spawn_node("Player");
        engine
            .active_scene
            .add_tag(player, TagComponent::new("player"))
            .expect("tag should attach");
        engine
            .active_scene
            .add_sprite(player, SpriteComponent::new("sprites/player.png"))
            .expect("sprite should attach");
        engine
            .active_scene
            .add_velocity(player, Velocity2DComponent::from_xy(120.0, 0.0))
            .expect("velocity should attach");
        engine
            .active_scene
            .add_collider(
                player,
                Collider2DComponent::rectangle(Vec2::new(16.0, 24.0)),
            )
            .expect("collider should attach");
        engine
            .active_scene
            .add_player_controller(player, PlayerControllerComponent::new(160.0))
            .expect("controller should attach");

        let camera = engine.active_scene.spawn_node("Camera2D");
        engine
            .active_scene
            .add_camera(camera, Camera2DComponent::default())
            .expect("camera should attach");
        engine
            .active_scene
            .add_camera_follow(camera, CameraFollowComponent::new(player))
            .expect("camera follow should attach");

        let trigger = engine.active_scene.spawn_node("CoinTrigger");
        engine
            .active_scene
            .add_trigger(trigger, TriggerComponent::new("coin").once())
            .expect("trigger should attach");

        let json = engine
            .project_document()
            .to_json_string()
            .expect("document should serialize");
        let loaded =
            ProjectDocument::from_json_str(&json).expect("document should deserialize from JSON");

        assert_eq!(loaded.project.name, "Saved Project");
        assert_eq!(loaded.assets.len(), 1);
        assert_eq!(loaded.active_scene.len(), 3);
        assert!(loaded.active_scene.find_node_by_tag("player").is_some());
        assert!(loaded.active_scene.find_node_by_name("Camera2D").is_some());
        assert!(loaded
            .active_scene
            .find_node_by_name("CoinTrigger")
            .is_some());
        assert_eq!(
            loaded
                .active_scene
                .velocity(player)
                .expect("velocity should load")
                .linear,
            Vec2::new(120.0, 0.0)
        );
        assert!(loaded.active_scene.collider(player).is_some());
        assert!(loaded.active_scene.player_controller(player).is_some());
        assert!(loaded.active_scene.camera_follow(camera).is_some());
        assert!(loaded.active_scene.trigger(trigger).is_some());
    }

    #[test]
    fn project_document_loads_scene_components_without_new_runtime_maps() {
        let json = r#"{
          "project": {
            "name": "Legacy Project",
            "root": null,
            "metadata": {
              "engine_version": "0.1.0",
              "philosophy_version": 1
            }
          },
          "assets": {
            "next_id": 0,
            "records": {}
          },
          "active_scene": {
            "name": "Main Scene",
            "next_id": 1,
            "nodes": [
              {
                "id": 0,
                "name": "Player",
                "transform": {
                  "position": { "x": 0.0, "y": 0.0 },
                  "rotation_radians": 0.0,
                  "scale": { "x": 1.0, "y": 1.0 }
                }
              }
            ],
            "components": {
              "tags": {
                "0": { "tag": "player" }
              },
              "sprites": {},
              "cameras": {}
            }
          }
        }"#;

        let document =
            ProjectDocument::from_json_str(json).expect("legacy project should deserialize");

        let player = document
            .active_scene
            .find_node_by_tag("player")
            .expect("player should load")
            .id;
        assert!(document.active_scene.tilemap(player).is_none());
        assert!(document.active_scene.velocity(player).is_none());
        assert!(document.active_scene.collider(player).is_none());
    }
}
