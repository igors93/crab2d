use std::error::Error;
use std::fmt;

use crab2d_core::Engine;
use crab2d_scene::{EntityId, Node2D, SceneError, SpriteComponent, TagComponent, Transform2D};

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

    /// Records a completed drag-move as a single undoable entry without re-applying
    /// the transform (the engine is already at `after` from live previews).
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
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandHistoryError {
    Command(EditorCommandError),
    NothingToUndo,
    NothingToRedo,
    Scene(SceneError),
    UnexpectedCommandResult,
}

impl fmt::Display for CommandHistoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Command(error) => write!(formatter, "{error}"),
            Self::NothingToUndo => formatter.write_str("there is no command to undo"),
            Self::NothingToRedo => formatter.write_str("there is no command to redo"),
            Self::Scene(error) => write!(formatter, "{error}"),
            Self::UnexpectedCommandResult => formatter.write_str("unexpected command result"),
        }
    }
}

impl Error for CommandHistoryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Command(error) => Some(error),
            Self::Scene(error) => Some(error),
            Self::NothingToUndo | Self::NothingToRedo | Self::UnexpectedCommandResult => None,
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

#[cfg(test)]
mod tests {
    use crab2d_core::{Engine, EngineConfig};
    use crab2d_scene::{SceneError, SpriteComponent, TagComponent, Transform2D, Vec2};

    use crate::{CommandHistory, CommandHistoryError, EditorCommand};

    #[test]
    fn create_node_can_be_undone_and_redone() {
        let mut engine = test_engine();
        let mut history = CommandHistory::default();

        let result = history
            .execute(EditorCommand::create_node("Enemy"), &mut engine)
            .expect("command should execute");
        let crate::EditorCommandResult::CreatedNode(enemy) = result else {
            panic!("create node should return an entity id");
        };

        assert!(engine.active_scene.node(enemy).is_some());
        assert!(history.can_undo());
        assert!(!history.can_redo());

        history.undo(&mut engine).expect("undo should succeed");

        assert!(engine.active_scene.node(enemy).is_none());
        assert!(!history.can_undo());
        assert!(history.can_redo());

        history.redo(&mut engine).expect("redo should succeed");

        assert!(engine.active_scene.find_node_by_name("Enemy").is_some());
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn rename_node_can_be_undone_and_redone() {
        let mut engine = test_engine();
        let mut history = CommandHistory::default();
        let player = engine.active_scene.spawn_node("Player");

        history
            .execute(EditorCommand::rename_node(player, "Hero"), &mut engine)
            .expect("rename should execute");

        assert_eq!(engine.active_scene.node(player).unwrap().name, "Hero");

        history.undo(&mut engine).expect("undo should succeed");

        assert_eq!(engine.active_scene.node(player).unwrap().name, "Player");

        history.redo(&mut engine).expect("redo should succeed");

        assert_eq!(engine.active_scene.node(player).unwrap().name, "Hero");
    }

    #[test]
    fn create_then_rename_can_be_undone_and_redone_in_order() {
        let mut engine = test_engine();
        let mut history = CommandHistory::default();

        let result = history
            .execute(EditorCommand::create_node("Enemy"), &mut engine)
            .expect("create should execute");
        let crate::EditorCommandResult::CreatedNode(enemy) = result else {
            panic!("create node should return an entity id");
        };

        history
            .execute(EditorCommand::rename_node(enemy, "Boss"), &mut engine)
            .expect("rename should execute");

        history
            .undo(&mut engine)
            .expect("rename undo should succeed");
        history
            .undo(&mut engine)
            .expect("create undo should succeed");
        assert!(engine.active_scene.node(enemy).is_none());

        history
            .redo(&mut engine)
            .expect("create redo should succeed");
        history
            .redo(&mut engine)
            .expect("rename redo should succeed");

        let node = engine.active_scene.node(enemy).expect("node should exist");
        assert_eq!(node.name, "Boss");
    }

    #[test]
    fn executing_command_clears_redo_stack() {
        let mut engine = test_engine();
        let mut history = CommandHistory::default();

        history
            .execute(EditorCommand::create_node("Enemy"), &mut engine)
            .expect("create should execute");
        history.undo(&mut engine).expect("undo should succeed");
        assert!(history.can_redo());

        history
            .execute(EditorCommand::create_node("Camera2D"), &mut engine)
            .expect("second create should execute");

        assert!(!history.can_redo());
    }

    #[test]
    fn move_node_can_be_undone_and_redone() {
        let mut engine = test_engine();
        let mut history = CommandHistory::default();
        let player = engine.active_scene.spawn_node("Player");
        let moved = Transform2D::from_position(Vec2::new(4.0, 4.0));

        history
            .execute(EditorCommand::move_node(player, moved), &mut engine)
            .expect("move should execute");

        assert_eq!(engine.active_scene.node(player).unwrap().transform, moved);

        history.undo(&mut engine).expect("undo should succeed");

        assert_eq!(
            engine.active_scene.node(player).unwrap().transform,
            Transform2D::default()
        );

        history.redo(&mut engine).expect("redo should succeed");

        assert_eq!(engine.active_scene.node(player).unwrap().transform, moved);
    }

    #[test]
    fn attach_tag_can_be_undone_and_redone() {
        let mut engine = test_engine();
        let mut history = CommandHistory::default();
        let player = engine.active_scene.spawn_node("Player");

        history
            .execute(EditorCommand::attach_tag(player, "player"), &mut engine)
            .expect("attach tag should execute");

        assert_eq!(engine.active_scene.tag(player).unwrap().tag, "player");

        history.undo(&mut engine).expect("undo should succeed");

        assert!(engine.active_scene.tag(player).is_none());

        history.redo(&mut engine).expect("redo should succeed");

        assert_eq!(engine.active_scene.tag(player).unwrap().tag, "player");
    }

    #[test]
    fn attach_tag_restores_previous_tag_on_undo() {
        let mut engine = test_engine();
        let mut history = CommandHistory::default();
        let player = engine.active_scene.spawn_node("Player");
        engine
            .active_scene
            .add_tag(player, TagComponent::new("old-player"))
            .expect("initial tag should attach");

        history
            .execute(EditorCommand::attach_tag(player, "player"), &mut engine)
            .expect("attach tag should execute");

        assert_eq!(engine.active_scene.tag(player).unwrap().tag, "player");

        history.undo(&mut engine).expect("undo should succeed");

        assert_eq!(engine.active_scene.tag(player).unwrap().tag, "old-player");
    }

    #[test]
    fn attach_sprite_restores_previous_sprite_on_undo_and_redo() {
        let mut engine = test_engine();
        let mut history = CommandHistory::default();
        let player = engine.active_scene.spawn_node("Player");
        engine
            .active_scene
            .add_sprite(
                player,
                SpriteComponent::new("sprites/old-player.png").with_z_index(7),
            )
            .expect("initial sprite should attach");

        history
            .execute(
                EditorCommand::attach_sprite(player, "sprites/player.png"),
                &mut engine,
            )
            .expect("attach sprite should execute");

        assert_eq!(
            engine.active_scene.sprite(player).unwrap().sprite_path,
            "sprites/player.png"
        );
        assert_eq!(engine.active_scene.sprite(player).unwrap().z_index, 0);

        history.undo(&mut engine).expect("undo should succeed");

        assert_eq!(
            engine.active_scene.sprite(player).unwrap().sprite_path,
            "sprites/old-player.png"
        );
        assert_eq!(engine.active_scene.sprite(player).unwrap().z_index, 7);

        history.redo(&mut engine).expect("redo should succeed");

        assert_eq!(
            engine.active_scene.sprite(player).unwrap().sprite_path,
            "sprites/player.png"
        );
    }

    #[test]
    fn undo_reports_when_history_is_empty() {
        let mut engine = test_engine();
        let mut history = CommandHistory::default();

        assert_eq!(
            history.undo(&mut engine),
            Err(CommandHistoryError::NothingToUndo)
        );
    }

    #[test]
    fn rename_missing_entity_reports_scene_error() {
        let mut engine = test_engine();
        let mut history = CommandHistory::default();

        let result = history.execute(
            EditorCommand::rename_node(crab2d_scene::EntityId::from_raw(999), "Missing"),
            &mut engine,
        );

        assert_eq!(
            result,
            Err(CommandHistoryError::Scene(SceneError::EntityNotFound))
        );
    }

    fn test_engine() -> Engine {
        Engine::new(EngineConfig::new("Crab2D Test"))
    }
}
