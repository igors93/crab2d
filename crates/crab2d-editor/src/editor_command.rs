use std::error::Error;
use std::fmt;

use crab2d_core::Engine;
use crab2d_scene::{
    EntityId, SceneError, SpriteComponent, TagComponent, TileCell, TilemapComponent, TilemapError,
    Transform2D,
};

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
    AttachTilemap {
        entity: EntityId,
        tilemap: TilemapComponent,
    },
    SetTile {
        entity: EntityId,
        layer_name: String,
        x: u32,
        y: u32,
        tile: Option<TileCell>,
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

    pub fn attach_tilemap(entity: EntityId, tilemap: TilemapComponent) -> Self {
        Self::AttachTilemap { entity, tilemap }
    }

    pub fn set_tile(
        entity: EntityId,
        layer_name: impl Into<String>,
        x: u32,
        y: u32,
        tile: Option<TileCell>,
    ) -> Self {
        Self::SetTile {
            entity,
            layer_name: layer_name.into(),
            x,
            y,
            tile,
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
            Self::AttachTilemap { entity, tilemap } => {
                engine.active_scene.add_tilemap(entity, tilemap)?;
                Ok(EditorCommandResult::None)
            }
            Self::SetTile {
                entity,
                layer_name,
                x,
                y,
                tile,
            } => {
                let tilemap = engine
                    .active_scene
                    .tilemap_mut(entity)
                    .ok_or(EditorCommandError::MissingTilemap)?;
                tilemap.set_tile(&layer_name, x, y, tile)?;
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
    MissingTilemap,
    Scene(SceneError),
    Tilemap(TilemapError),
}

impl fmt::Display for EditorCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingTilemap => formatter.write_str("tilemap component was not found"),
            Self::Scene(error) => write!(formatter, "{error}"),
            Self::Tilemap(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for EditorCommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MissingTilemap => None,
            Self::Scene(error) => Some(error),
            Self::Tilemap(error) => Some(error),
        }
    }
}

impl From<SceneError> for EditorCommandError {
    fn from(error: SceneError) -> Self {
        Self::Scene(error)
    }
}

impl From<TilemapError> for EditorCommandError {
    fn from(error: TilemapError) -> Self {
        Self::Tilemap(error)
    }
}

#[cfg(test)]
mod tests {
    use crab2d_core::{Engine, EngineConfig};
    use crab2d_scene::{SceneError, TileCell, TileSize, TilemapComponent, TilemapSize};

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
    fn set_tile_updates_existing_tilemap() {
        let mut engine = test_engine();
        let world = engine.active_scene.spawn_node("World");
        engine
            .active_scene
            .add_tilemap(world, test_tilemap())
            .expect("tilemap should attach");

        EditorCommand::set_tile(world, "Ground", 1, 1, Some(TileCell::new(4)))
            .apply(&mut engine)
            .expect("tile should paint");

        assert_eq!(
            engine
                .active_scene
                .tilemap(world)
                .expect("tilemap exists")
                .tile("Ground", 1, 1),
            Ok(Some(TileCell::new(4)))
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

    fn test_tilemap() -> TilemapComponent {
        TilemapComponent::new(TilemapSize::new(4, 4), TileSize::new(32, 32))
            .expect("tilemap should be valid")
    }
}
