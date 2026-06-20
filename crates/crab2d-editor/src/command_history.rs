use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

use crab2d_core::Engine;
use crab2d_scene::{
    CameraFollowComponent, Collider2DComponent, EntityId, Node2D, PlayerControllerComponent,
    SceneError, SpriteComponent, TagComponent, TileCell, TilemapComponent, TilemapError,
    Transform2D, TriggerComponent, Velocity2DComponent,
};

use crate::{EditorCommand, EditorCommandError, EditorCommandResult};

#[derive(Debug, Default)]
pub struct CommandHistory {
    undo_stack: Vec<AppliedEditorCommand>,
    redo_stack: Vec<AppliedEditorCommand>,
}

impl CommandHistory {
    pub fn execute(
        &mut self,
        command: EditorCommand,
        engine: &mut Engine,
    ) -> Result<EditorCommandResult, CommandHistoryError> {
        let applied = AppliedEditorCommand::from_command(&command, engine)?;
        let result = command.apply(engine)?;

        self.undo_stack.push(applied.after_apply(&result, engine)?);
        self.redo_stack.clear();

        Ok(result)
    }

    pub fn undo(&mut self, engine: &mut Engine) -> Result<(), CommandHistoryError> {
        let applied = self
            .undo_stack
            .pop()
            .ok_or(CommandHistoryError::NothingToUndo)?;
        if let Err(error) = applied.undo(engine) {
            self.undo_stack.push(applied);
            return Err(error);
        }
        self.redo_stack.push(applied);
        Ok(())
    }

    pub fn redo(&mut self, engine: &mut Engine) -> Result<(), CommandHistoryError> {
        let applied = self
            .redo_stack
            .pop()
            .ok_or(CommandHistoryError::NothingToRedo)?;
        let redone = match applied.clone().redo(engine) {
            Ok(redone) => redone,
            Err(error) => {
                self.redo_stack.push(applied);
                return Err(error);
            }
        };
        self.undo_stack.push(redone);
        Ok(())
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn push_move_node(&mut self, entity: EntityId, before: Transform2D, after: Transform2D) {
        if before == after {
            return;
        }
        self.undo_stack.push(AppliedEditorCommand::MoveNode {
            entity,
            before,
            after,
        });
        self.redo_stack.clear();
    }
}

#[derive(Debug, Clone, PartialEq)]
enum AppliedEditorCommand {
    CreateNode {
        name: String,
        node: Option<Node2D>,
    },
    RenameNode {
        entity: EntityId,
        before: String,
        after: String,
    },
    MoveNode {
        entity: EntityId,
        before: Transform2D,
        after: Transform2D,
    },
    AttachTag {
        entity: EntityId,
        before: Option<TagComponent>,
        after: TagComponent,
    },
    AttachSprite {
        entity: EntityId,
        before: Option<SpriteComponent>,
        after: SpriteComponent,
    },
    AttachTilemap {
        entity: EntityId,
        before: Option<TilemapComponent>,
        after: TilemapComponent,
    },
    AttachVelocity {
        entity: EntityId,
        before: Option<Velocity2DComponent>,
        after: Velocity2DComponent,
    },
    AttachCollider {
        entity: EntityId,
        before: Option<Collider2DComponent>,
        after: Collider2DComponent,
    },
    AttachPlayerController {
        entity: EntityId,
        before: Option<PlayerControllerComponent>,
        after: PlayerControllerComponent,
    },
    AttachCameraFollow {
        entity: EntityId,
        before: Option<CameraFollowComponent>,
        after: CameraFollowComponent,
    },
    AttachTrigger {
        entity: EntityId,
        before: Option<TriggerComponent>,
        after: TriggerComponent,
    },
    SetTileCollision {
        entity: EntityId,
        before: BTreeSet<u32>,
        after: BTreeSet<u32>,
    },
    SetTile {
        entity: EntityId,
        layer_name: String,
        x: u32,
        y: u32,
        before: Option<TileCell>,
        after: Option<TileCell>,
    },
}

impl AppliedEditorCommand {
    fn from_command(command: &EditorCommand, engine: &Engine) -> Result<Self, CommandHistoryError> {
        match command {
            EditorCommand::CreateNode { name } => Ok(Self::CreateNode {
                name: name.clone(),
                node: None,
            }),
            EditorCommand::RenameNode { entity, name } => {
                let before = engine
                    .active_scene
                    .node(*entity)
                    .ok_or(SceneError::EntityNotFound)?
                    .name
                    .clone();

                Ok(Self::RenameNode {
                    entity: *entity,
                    before,
                    after: name.clone(),
                })
            }
            EditorCommand::MoveNode { entity, transform } => {
                let before = engine
                    .active_scene
                    .node(*entity)
                    .ok_or(SceneError::EntityNotFound)?
                    .transform;

                Ok(Self::MoveNode {
                    entity: *entity,
                    before,
                    after: *transform,
                })
            }
            EditorCommand::AttachTag { entity, tag } => {
                engine
                    .active_scene
                    .node(*entity)
                    .ok_or(SceneError::EntityNotFound)?;

                Ok(Self::AttachTag {
                    entity: *entity,
                    before: engine.active_scene.tag(*entity).cloned(),
                    after: TagComponent::new(tag.clone()),
                })
            }
            EditorCommand::AttachSprite {
                entity,
                sprite_path,
            } => {
                engine
                    .active_scene
                    .node(*entity)
                    .ok_or(SceneError::EntityNotFound)?;

                Ok(Self::AttachSprite {
                    entity: *entity,
                    before: engine.active_scene.sprite(*entity).cloned(),
                    after: SpriteComponent::new(sprite_path.clone()),
                })
            }
            EditorCommand::AttachTilemap { entity, tilemap } => {
                engine
                    .active_scene
                    .node(*entity)
                    .ok_or(SceneError::EntityNotFound)?;

                Ok(Self::AttachTilemap {
                    entity: *entity,
                    before: engine.active_scene.tilemap(*entity).cloned(),
                    after: tilemap.clone(),
                })
            }
            EditorCommand::AttachVelocity { entity, velocity } => {
                engine
                    .active_scene
                    .node(*entity)
                    .ok_or(SceneError::EntityNotFound)?;

                Ok(Self::AttachVelocity {
                    entity: *entity,
                    before: engine.active_scene.velocity(*entity).copied(),
                    after: *velocity,
                })
            }
            EditorCommand::AttachCollider { entity, collider } => {
                engine
                    .active_scene
                    .node(*entity)
                    .ok_or(SceneError::EntityNotFound)?;

                Ok(Self::AttachCollider {
                    entity: *entity,
                    before: engine.active_scene.collider(*entity).copied(),
                    after: *collider,
                })
            }
            EditorCommand::AttachPlayerController { entity, controller } => {
                engine
                    .active_scene
                    .node(*entity)
                    .ok_or(SceneError::EntityNotFound)?;

                Ok(Self::AttachPlayerController {
                    entity: *entity,
                    before: engine.active_scene.player_controller(*entity).copied(),
                    after: *controller,
                })
            }
            EditorCommand::AttachCameraFollow { entity, follow } => {
                engine
                    .active_scene
                    .node(*entity)
                    .ok_or(SceneError::EntityNotFound)?;

                Ok(Self::AttachCameraFollow {
                    entity: *entity,
                    before: engine.active_scene.camera_follow(*entity).copied(),
                    after: *follow,
                })
            }
            EditorCommand::AttachTrigger { entity, trigger } => {
                engine
                    .active_scene
                    .node(*entity)
                    .ok_or(SceneError::EntityNotFound)?;

                Ok(Self::AttachTrigger {
                    entity: *entity,
                    before: engine.active_scene.trigger(*entity).cloned(),
                    after: trigger.clone(),
                })
            }
            EditorCommand::SetTileCollision {
                entity,
                solid_tiles,
            } => {
                let before = engine
                    .active_scene
                    .tilemap(*entity)
                    .ok_or(CommandHistoryError::MissingTilemap)?
                    .collision
                    .solid_tiles
                    .clone();

                Ok(Self::SetTileCollision {
                    entity: *entity,
                    before,
                    after: solid_tiles.clone(),
                })
            }
            EditorCommand::SetTile {
                entity,
                layer_name,
                x,
                y,
                tile,
            } => {
                let before = engine
                    .active_scene
                    .tilemap(*entity)
                    .ok_or(CommandHistoryError::MissingTilemap)?
                    .tile(layer_name, *x, *y)?;

                Ok(Self::SetTile {
                    entity: *entity,
                    layer_name: layer_name.clone(),
                    x: *x,
                    y: *y,
                    before,
                    after: *tile,
                })
            }
        }
    }

    fn after_apply(
        self,
        result: &EditorCommandResult,
        engine: &Engine,
    ) -> Result<Self, CommandHistoryError> {
        match (self, result) {
            (Self::CreateNode { name, node: None }, EditorCommandResult::CreatedNode(entity)) => {
                let node = engine
                    .active_scene
                    .node(*entity)
                    .ok_or(SceneError::EntityNotFound)?
                    .clone();
                Ok(Self::CreateNode {
                    name,
                    node: Some(node),
                })
            }
            (
                Self::RenameNode {
                    entity,
                    before,
                    after,
                },
                EditorCommandResult::None,
            ) => Ok(Self::RenameNode {
                entity,
                before,
                after,
            }),
            (
                Self::MoveNode {
                    entity,
                    before,
                    after,
                },
                EditorCommandResult::None,
            ) => Ok(Self::MoveNode {
                entity,
                before,
                after,
            }),
            (
                Self::AttachTag {
                    entity,
                    before,
                    after,
                },
                EditorCommandResult::None,
            ) => Ok(Self::AttachTag {
                entity,
                before,
                after,
            }),
            (
                Self::AttachSprite {
                    entity,
                    before,
                    after,
                },
                EditorCommandResult::None,
            ) => Ok(Self::AttachSprite {
                entity,
                before,
                after,
            }),
            (
                Self::AttachTilemap {
                    entity,
                    before,
                    after,
                },
                EditorCommandResult::None,
            ) => Ok(Self::AttachTilemap {
                entity,
                before,
                after,
            }),
            (
                Self::AttachVelocity {
                    entity,
                    before,
                    after,
                },
                EditorCommandResult::None,
            ) => Ok(Self::AttachVelocity {
                entity,
                before,
                after,
            }),
            (
                Self::AttachCollider {
                    entity,
                    before,
                    after,
                },
                EditorCommandResult::None,
            ) => Ok(Self::AttachCollider {
                entity,
                before,
                after,
            }),
            (
                Self::AttachPlayerController {
                    entity,
                    before,
                    after,
                },
                EditorCommandResult::None,
            ) => Ok(Self::AttachPlayerController {
                entity,
                before,
                after,
            }),
            (
                Self::AttachCameraFollow {
                    entity,
                    before,
                    after,
                },
                EditorCommandResult::None,
            ) => Ok(Self::AttachCameraFollow {
                entity,
                before,
                after,
            }),
            (
                Self::AttachTrigger {
                    entity,
                    before,
                    after,
                },
                EditorCommandResult::None,
            ) => Ok(Self::AttachTrigger {
                entity,
                before,
                after,
            }),
            (
                Self::SetTileCollision {
                    entity,
                    before,
                    after,
                },
                EditorCommandResult::None,
            ) => Ok(Self::SetTileCollision {
                entity,
                before,
                after,
            }),
            (
                Self::SetTile {
                    entity,
                    layer_name,
                    x,
                    y,
                    before,
                    after,
                },
                EditorCommandResult::None,
            ) => Ok(Self::SetTile {
                entity,
                layer_name,
                x,
                y,
                before,
                after,
            }),
            _ => Err(CommandHistoryError::UnexpectedCommandResult),
        }
    }

    fn undo(&self, engine: &mut Engine) -> Result<(), CommandHistoryError> {
        match self {
            Self::CreateNode {
                node: Some(node), ..
            } => {
                engine.active_scene.despawn_node(node.id)?;
                Ok(())
            }
            Self::CreateNode { node: None, .. } => {
                Err(CommandHistoryError::UnexpectedCommandResult)
            }
            Self::RenameNode { entity, before, .. } => {
                let node = engine
                    .active_scene
                    .node_mut(*entity)
                    .ok_or(SceneError::EntityNotFound)?;
                node.name = before.clone();
                Ok(())
            }
            Self::MoveNode { entity, before, .. } => {
                let node = engine
                    .active_scene
                    .node_mut(*entity)
                    .ok_or(SceneError::EntityNotFound)?;
                node.transform = *before;
                Ok(())
            }
            Self::AttachTag { entity, before, .. } => {
                if let Some(component) = before {
                    engine.active_scene.add_tag(*entity, component.clone())?;
                } else {
                    engine.active_scene.remove_tag(*entity)?;
                }
                Ok(())
            }
            Self::AttachSprite { entity, before, .. } => {
                if let Some(component) = before {
                    engine.active_scene.add_sprite(*entity, component.clone())?;
                } else {
                    engine.active_scene.remove_sprite(*entity)?;
                }
                Ok(())
            }
            Self::AttachTilemap { entity, before, .. } => {
                if let Some(component) = before {
                    engine
                        .active_scene
                        .add_tilemap(*entity, component.clone())?;
                } else {
                    engine.active_scene.remove_tilemap(*entity)?;
                }
                Ok(())
            }
            Self::AttachVelocity { entity, before, .. } => {
                if let Some(component) = before {
                    engine.active_scene.add_velocity(*entity, *component)?;
                } else {
                    engine.active_scene.remove_velocity(*entity)?;
                }
                Ok(())
            }
            Self::AttachCollider { entity, before, .. } => {
                if let Some(component) = before {
                    engine.active_scene.add_collider(*entity, *component)?;
                } else {
                    engine.active_scene.remove_collider(*entity)?;
                }
                Ok(())
            }
            Self::AttachPlayerController { entity, before, .. } => {
                if let Some(component) = before {
                    engine
                        .active_scene
                        .add_player_controller(*entity, *component)?;
                } else {
                    engine.active_scene.remove_player_controller(*entity)?;
                }
                Ok(())
            }
            Self::AttachCameraFollow { entity, before, .. } => {
                if let Some(component) = before {
                    engine.active_scene.add_camera_follow(*entity, *component)?;
                } else {
                    engine.active_scene.remove_camera_follow(*entity)?;
                }
                Ok(())
            }
            Self::AttachTrigger { entity, before, .. } => {
                if let Some(component) = before {
                    engine
                        .active_scene
                        .add_trigger(*entity, component.clone())?;
                } else {
                    engine.active_scene.remove_trigger(*entity)?;
                }
                Ok(())
            }
            Self::SetTileCollision { entity, before, .. } => {
                engine
                    .active_scene
                    .tilemap_mut(*entity)
                    .ok_or(CommandHistoryError::MissingTilemap)?
                    .collision
                    .solid_tiles = before.clone();
                Ok(())
            }
            Self::SetTile {
                entity,
                layer_name,
                x,
                y,
                before,
                ..
            } => {
                engine
                    .active_scene
                    .tilemap_mut(*entity)
                    .ok_or(CommandHistoryError::MissingTilemap)?
                    .set_tile(layer_name, *x, *y, *before)?;
                Ok(())
            }
        }
    }

    fn redo(self, engine: &mut Engine) -> Result<Self, CommandHistoryError> {
        match self {
            Self::CreateNode {
                name,
                node: Some(node),
            } => {
                engine.active_scene.restore_node(node.clone())?;
                Ok(Self::CreateNode {
                    name,
                    node: Some(node),
                })
            }
            Self::CreateNode { node: None, .. } => {
                Err(CommandHistoryError::UnexpectedCommandResult)
            }
            Self::RenameNode {
                entity,
                before,
                after,
            } => {
                let node = engine
                    .active_scene
                    .node_mut(entity)
                    .ok_or(SceneError::EntityNotFound)?;
                node.name = after.clone();
                Ok(Self::RenameNode {
                    entity,
                    before,
                    after,
                })
            }
            Self::MoveNode {
                entity,
                before,
                after,
            } => {
                let node = engine
                    .active_scene
                    .node_mut(entity)
                    .ok_or(SceneError::EntityNotFound)?;
                node.transform = after;
                Ok(Self::MoveNode {
                    entity,
                    before,
                    after,
                })
            }
            Self::AttachTag {
                entity,
                before,
                after,
            } => {
                engine.active_scene.add_tag(entity, after.clone())?;
                Ok(Self::AttachTag {
                    entity,
                    before,
                    after,
                })
            }
            Self::AttachSprite {
                entity,
                before,
                after,
            } => {
                engine.active_scene.add_sprite(entity, after.clone())?;
                Ok(Self::AttachSprite {
                    entity,
                    before,
                    after,
                })
            }
            Self::AttachTilemap {
                entity,
                before,
                after,
            } => {
                engine.active_scene.add_tilemap(entity, after.clone())?;
                Ok(Self::AttachTilemap {
                    entity,
                    before,
                    after,
                })
            }
            Self::AttachVelocity {
                entity,
                before,
                after,
            } => {
                engine.active_scene.add_velocity(entity, after)?;
                Ok(Self::AttachVelocity {
                    entity,
                    before,
                    after,
                })
            }
            Self::AttachCollider {
                entity,
                before,
                after,
            } => {
                engine.active_scene.add_collider(entity, after)?;
                Ok(Self::AttachCollider {
                    entity,
                    before,
                    after,
                })
            }
            Self::AttachPlayerController {
                entity,
                before,
                after,
            } => {
                engine.active_scene.add_player_controller(entity, after)?;
                Ok(Self::AttachPlayerController {
                    entity,
                    before,
                    after,
                })
            }
            Self::AttachCameraFollow {
                entity,
                before,
                after,
            } => {
                engine.active_scene.add_camera_follow(entity, after)?;
                Ok(Self::AttachCameraFollow {
                    entity,
                    before,
                    after,
                })
            }
            Self::AttachTrigger {
                entity,
                before,
                after,
            } => {
                engine.active_scene.add_trigger(entity, after.clone())?;
                Ok(Self::AttachTrigger {
                    entity,
                    before,
                    after,
                })
            }
            Self::SetTileCollision {
                entity,
                before,
                after,
            } => {
                engine
                    .active_scene
                    .tilemap_mut(entity)
                    .ok_or(CommandHistoryError::MissingTilemap)?
                    .collision
                    .solid_tiles = after.clone();
                Ok(Self::SetTileCollision {
                    entity,
                    before,
                    after,
                })
            }
            Self::SetTile {
                entity,
                layer_name,
                x,
                y,
                before,
                after,
            } => {
                engine
                    .active_scene
                    .tilemap_mut(entity)
                    .ok_or(CommandHistoryError::MissingTilemap)?
                    .set_tile(&layer_name, x, y, after)?;
                Ok(Self::SetTile {
                    entity,
                    layer_name,
                    x,
                    y,
                    before,
                    after,
                })
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandHistoryError {
    Command(EditorCommandError),
    MissingTilemap,
    NothingToUndo,
    NothingToRedo,
    Scene(SceneError),
    Tilemap(TilemapError),
    UnexpectedCommandResult,
}

impl fmt::Display for CommandHistoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Command(error) => write!(formatter, "{error}"),
            Self::MissingTilemap => formatter.write_str("tilemap component was not found"),
            Self::NothingToUndo => formatter.write_str("there is no command to undo"),
            Self::NothingToRedo => formatter.write_str("there is no command to redo"),
            Self::Scene(error) => write!(formatter, "{error}"),
            Self::Tilemap(error) => write!(formatter, "{error}"),
            Self::UnexpectedCommandResult => formatter.write_str("unexpected command result"),
        }
    }
}

impl Error for CommandHistoryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Command(error) => Some(error),
            Self::Scene(error) => Some(error),
            Self::Tilemap(error) => Some(error),
            Self::MissingTilemap
            | Self::NothingToUndo
            | Self::NothingToRedo
            | Self::UnexpectedCommandResult => None,
        }
    }
}

impl From<EditorCommandError> for CommandHistoryError {
    fn from(error: EditorCommandError) -> Self {
        Self::Command(error)
    }
}

impl From<SceneError> for CommandHistoryError {
    fn from(error: SceneError) -> Self {
        Self::Scene(error)
    }
}

impl From<TilemapError> for CommandHistoryError {
    fn from(error: TilemapError) -> Self {
        Self::Tilemap(error)
    }
}

#[cfg(test)]
mod tests {
    use crab2d_core::{Engine, EngineConfig};
    use crab2d_scene::{TileCell, TileSize, TilemapComponent, TilemapSize};

    use crate::{CommandHistory, EditorCommand};

    #[test]
    fn set_tile_can_be_undone_and_redone() {
        let mut engine = test_engine();
        let mut history = CommandHistory::default();
        let world = engine.active_scene.spawn_node("World");
        engine
            .active_scene
            .add_tilemap(world, test_tilemap())
            .expect("tilemap should attach");

        history
            .execute(
                EditorCommand::set_tile(world, "Ground", 1, 1, Some(TileCell::new(2))),
                &mut engine,
            )
            .expect("paint should execute");

        assert_eq!(
            engine
                .active_scene
                .tilemap(world)
                .expect("tilemap exists")
                .tile("Ground", 1, 1),
            Ok(Some(TileCell::new(2)))
        );

        history.undo(&mut engine).expect("undo should succeed");
        assert_eq!(
            engine
                .active_scene
                .tilemap(world)
                .expect("tilemap exists")
                .tile("Ground", 1, 1),
            Ok(None)
        );

        history.redo(&mut engine).expect("redo should succeed");
        assert_eq!(
            engine
                .active_scene
                .tilemap(world)
                .expect("tilemap exists")
                .tile("Ground", 1, 1),
            Ok(Some(TileCell::new(2)))
        );
    }

    #[test]
    fn attach_tilemap_restores_previous_tilemap_on_undo() {
        let mut engine = test_engine();
        let mut history = CommandHistory::default();
        let world = engine.active_scene.spawn_node("World");

        history
            .execute(
                EditorCommand::attach_tilemap(world, test_tilemap()),
                &mut engine,
            )
            .expect("tilemap attach should execute");
        assert!(engine.active_scene.tilemap(world).is_some());

        history.undo(&mut engine).expect("undo should succeed");
        assert!(engine.active_scene.tilemap(world).is_none());
    }

    fn test_engine() -> Engine {
        Engine::new(EngineConfig::new("Crab2D Test"))
    }

    fn test_tilemap() -> TilemapComponent {
        TilemapComponent::new(TilemapSize::new(4, 4), TileSize::new(32, 32))
            .expect("tilemap should be valid")
    }
}
