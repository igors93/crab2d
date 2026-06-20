use std::error::Error;
use std::fmt;

use crab2d_core::Engine;
use crab2d_scene::{EntityId, SceneError, SpriteComponent, TagComponent, Transform2D};

#[derive(Debug, Clone, PartialEq)]
pub enum EditorCommand {
    CreateNode {
        name: String,
    },
    RenameNode {
        entity: EntityId,
        name: String,
    },
    MoveNode {
        entity: EntityId,
        transform: Transform2D,
    },
    AttachTag {
        entity: EntityId,
        tag: String,
    },
    AttachSprite {
        entity: EntityId,
        sprite_path: String,
    },
}

impl EditorCommand {
    pub fn create_node(name: impl Into<String>) -> Self {
        Self::CreateNode { name: name.into() }
    }

    pub fn rename_node(entity: EntityId, name: impl Into<String>) -> Self {
        Self::RenameNode {
            entity,
            name: name.into(),
        }
    }

    pub fn move_node(entity: EntityId, transform: Transform2D) -> Self {
        Self::MoveNode { entity, transform }
    }

    pub fn attach_tag(entity: EntityId, tag: impl Into<String>) -> Self {
        Self::AttachTag {
            entity,
            tag: tag.into(),
        }
    }

    pub fn attach_sprite(entity: EntityId, sprite_path: impl Into<String>) -> Self {
        Self::AttachSprite {
            entity,
            sprite_path: sprite_path.into(),
        }
    }

    pub fn apply(self, engine: &mut Engine) -> Result<EditorCommandResult, EditorCommandError> {
        match self {
            Self::CreateNode { name } => {
                let entity = engine.active_scene.try_spawn_node(name)?;
                Ok(EditorCommandResult::CreatedNode(entity))
            }
            Self::RenameNode { entity, name } => {
                if name.is_empty() {
                    return Err(SceneError::EmptyNodeName.into());
                }

                let node = engine
                    .active_scene
                    .node_mut(entity)
                    .ok_or(SceneError::EntityNotFound)?;
                node.name = name;
                Ok(EditorCommandResult::None)
            }
            Self::MoveNode { entity, transform } => {
                if !transform.is_finite() {
                    return Err(SceneError::InvalidTransform.into());
                }

                let node = engine
                    .active_scene
                    .node_mut(entity)
                    .ok_or(SceneError::EntityNotFound)?;
                node.transform = transform;
                Ok(EditorCommandResult::None)
            }
            Self::AttachTag { entity, tag } => {
                engine
                    .active_scene
                    .add_tag(entity, TagComponent::new(tag))?;
                Ok(EditorCommandResult::None)
            }
            Self::AttachSprite {
                entity,
                sprite_path,
            } => {
                engine
                    .active_scene
                    .add_sprite(entity, SpriteComponent::new(sprite_path))?;
                Ok(EditorCommandResult::None)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorCommandResult {
    None,
    CreatedNode(EntityId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorCommandError {
    Scene(SceneError),
}

impl fmt::Display for EditorCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scene(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for EditorCommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Scene(error) => Some(error),
        }
    }
}

impl From<SceneError> for EditorCommandError {
    fn from(error: SceneError) -> Self {
        Self::Scene(error)
    }
}

#[cfg(test)]
mod tests {
    use crab2d_core::{Engine, EngineConfig};
    use crab2d_scene::{SceneError, Transform2D, Vec2};

    use crate::{EditorCommand, EditorCommandError, EditorCommandResult};

    #[test]
    fn create_node_adds_node_to_active_scene() {
        let mut engine = test_engine();

        let result = EditorCommand::create_node("Enemy")
            .apply(&mut engine)
            .expect("command should succeed");

        let EditorCommandResult::CreatedNode(entity) = result else {
            panic!("create node should return the created entity id");
        };
        assert_eq!(
            engine.active_scene.node(entity).expect("node exists").name,
            "Enemy"
        );
    }

    #[test]
    fn rename_node_updates_existing_node() {
        let mut engine = test_engine();
        let player = engine.active_scene.spawn_node("Player");

        EditorCommand::rename_node(player, "Hero")
            .apply(&mut engine)
            .expect("command should succeed");

        assert_eq!(
            engine.active_scene.node(player).expect("node exists").name,
            "Hero"
        );
    }

    #[test]
    fn move_node_updates_transform() {
        let mut engine = test_engine();
        let player = engine.active_scene.spawn_node("Player");
        let transform = Transform2D::from_position(Vec2::new(12.0, 8.0));

        EditorCommand::move_node(player, transform)
            .apply(&mut engine)
            .expect("command should succeed");

        assert_eq!(
            engine
                .active_scene
                .node(player)
                .expect("node exists")
                .transform,
            transform
        );
    }

    #[test]
    fn attach_tag_adds_tag_component() {
        let mut engine = test_engine();
        let player = engine.active_scene.spawn_node("Player");

        EditorCommand::attach_tag(player, "player")
            .apply(&mut engine)
            .expect("command should succeed");

        assert_eq!(
            engine.active_scene.tag(player).expect("tag exists").tag,
            "player"
        );
    }

    #[test]
    fn attach_sprite_adds_sprite_component() {
        let mut engine = test_engine();
        let player = engine.active_scene.spawn_node("Player");

        EditorCommand::attach_sprite(player, "sprites/player.png")
            .apply(&mut engine)
            .expect("command should succeed");

        assert_eq!(
            engine
                .active_scene
                .sprite(player)
                .expect("sprite exists")
                .sprite_path,
            "sprites/player.png"
        );
    }

    #[test]
    fn command_reports_scene_errors() {
        let mut engine = test_engine();
        let result = EditorCommand::create_node("").apply(&mut engine);

        assert_eq!(
            result,
            Err(EditorCommandError::Scene(SceneError::EmptyNodeName))
        );
        assert!(engine.active_scene.is_empty());
    }

    fn test_engine() -> Engine {
        Engine::new(EngineConfig::new("Crab2D Test"))
    }
}
