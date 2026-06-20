use std::error::Error;
use std::fmt;

use crab2d_core::Engine;
use crab2d_scene::{EntityId, Node2D, SceneError};

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
            _ => Err(CommandHistoryError::UnsupportedCommand),
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
    UnsupportedCommand,
}

impl fmt::Display for CommandHistoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Command(error) => write!(formatter, "{error}"),
            Self::NothingToUndo => formatter.write_str("there is no command to undo"),
            Self::NothingToRedo => formatter.write_str("there is no command to redo"),
            Self::Scene(error) => write!(formatter, "{error}"),
            Self::UnexpectedCommandResult => formatter.write_str("unexpected command result"),
            Self::UnsupportedCommand => formatter.write_str("command is not tracked by history"),
        }
    }
}

impl Error for CommandHistoryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Command(error) => Some(error),
            Self::Scene(error) => Some(error),
            Self::NothingToUndo
            | Self::NothingToRedo
            | Self::UnexpectedCommandResult
            | Self::UnsupportedCommand => None,
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
    use crab2d_scene::{SceneError, Transform2D, Vec2};

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
    fn unsupported_commands_are_not_applied_by_history() {
        let mut engine = test_engine();
        let mut history = CommandHistory::default();
        let player = engine.active_scene.spawn_node("Player");

        let result = history.execute(
            EditorCommand::move_node(player, Transform2D::from_position(Vec2::new(4.0, 4.0))),
            &mut engine,
        );

        assert_eq!(result, Err(CommandHistoryError::UnsupportedCommand));
        assert_eq!(
            engine.active_scene.node(player).unwrap().transform,
            Transform2D::default()
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
