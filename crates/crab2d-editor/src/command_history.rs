use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

use crab2d_core::Engine;
use crab2d_scene::{
    Camera2DComponent, CameraFollowComponent, Collider2DComponent, EntityId, Node2D,
    PlayerControllerComponent, PrefabTemplate, SceneError, SpriteComponent, TagComponent, TileCell,
    TilemapComponent, TilemapError, Transform2D, TriggerComponent, Velocity2DComponent,
};

use crate::{
    EditorCommand, EditorCommandError, EditorCommandResult, EditorComponentKind,
    NodeComponentSnapshot,
};

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

    pub fn push_attach_camera(
        &mut self,
        entity: EntityId,
        before: Option<Camera2DComponent>,
        after: Camera2DComponent,
    ) {
        if before == Some(after) {
            return;
        }
        self.undo_stack.push(AppliedEditorCommand::AttachCamera {
            entity,
            before,
            after,
        });
        self.redo_stack.clear();
    }

    pub fn push_attach_collider(
        &mut self,
        entity: EntityId,
        before: Option<Collider2DComponent>,
        after: Collider2DComponent,
    ) {
        if before == Some(after) {
            return;
        }
        self.undo_stack.push(AppliedEditorCommand::AttachCollider {
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
    DeleteNode {
        node: Node2D,
        components: NodeComponentSnapshot,
    },
    DuplicateNode {
        source: EntityId,
        node: Option<Node2D>,
        components: Option<NodeComponentSnapshot>,
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
    AttachCamera {
        entity: EntityId,
        before: Option<Camera2DComponent>,
        after: Camera2DComponent,
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
    RemoveComponent {
        entity: EntityId,
        before: NodeComponentSnapshot,
        after: Option<NodeComponentSnapshot>,
    },
    ApplyGameplayPreset {
        entity: EntityId,
        before: NodeComponentSnapshot,
        after: Option<NodeComponentSnapshot>,
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
    /// Covers CreateFromAsset, CreateCamera, CreateWorldTextNode, CreateScenePortal,
    /// and InstantiatePrefab — all commands that spawn a single node.
    SpawnedNode {
        node: Option<Node2D>,
        components: Option<NodeComponentSnapshot>,
    },
    /// Covers CreatePrefabFromEntity — records the replaced prefab, if any,
    /// and the template registered by the command.
    RegisteredPrefab {
        prefab_name: String,
        previous: Option<PrefabTemplate>,
        registered: Option<PrefabTemplate>,
    },
    /// Covers RemovePrefab — stores the template so it can be restored on undo.
    RemovedPrefab {
        template: Option<PrefabTemplate>,
    },
}

impl AppliedEditorCommand {
    fn from_command(command: &EditorCommand, engine: &Engine) -> Result<Self, CommandHistoryError> {
        match command {
            EditorCommand::CreateNode { name } => Ok(Self::CreateNode {
                name: name.clone(),
                node: None,
            }),
            EditorCommand::DeleteNode { entity } => {
                let node = engine
                    .active_scene
                    .node(*entity)
                    .ok_or(SceneError::EntityNotFound)?
                    .clone();
                let components = NodeComponentSnapshot::capture(engine, *entity)?;
                Ok(Self::DeleteNode { node, components })
            }
            EditorCommand::DuplicateNode { entity } => {
                engine
                    .active_scene
                    .node(*entity)
                    .ok_or(SceneError::EntityNotFound)?;
                Ok(Self::DuplicateNode {
                    source: *entity,
                    node: None,
                    components: None,
                })
            }
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
            EditorCommand::AttachCamera { entity, camera } => {
                engine
                    .active_scene
                    .node(*entity)
                    .ok_or(SceneError::EntityNotFound)?;

                Ok(Self::AttachCamera {
                    entity: *entity,
                    before: engine.active_scene.camera(*entity).copied(),
                    after: *camera,
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
            EditorCommand::RemoveComponent { entity, component } => {
                let before = NodeComponentSnapshot::capture(engine, *entity)?;
                if !snapshot_has_component(&before, *component) {
                    return Err(CommandHistoryError::MissingComponent);
                }
                Ok(Self::RemoveComponent {
                    entity: *entity,
                    before,
                    after: None,
                })
            }
            EditorCommand::ApplyGameplayPreset { entity, .. } => Ok(Self::ApplyGameplayPreset {
                entity: *entity,
                before: NodeComponentSnapshot::capture(engine, *entity)?,
                after: None,
            }),
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
            EditorCommand::CreateFromAsset { .. }
            | EditorCommand::CreateCamera { .. }
            | EditorCommand::CreateWorldTextNode { .. }
            | EditorCommand::CreateScenePortal { .. }
            | EditorCommand::InstantiatePrefab { .. } => Ok(Self::SpawnedNode {
                node: None,
                components: None,
            }),
            EditorCommand::CreatePrefabFromEntity { prefab_name, .. } => {
                let previous = engine.prefabs.get(prefab_name).cloned();
                Ok(Self::RegisteredPrefab {
                    prefab_name: prefab_name.clone(),
                    previous,
                    registered: None,
                })
            }
            EditorCommand::RemovePrefab { prefab_name } => {
                let template = engine.prefabs.get(prefab_name).cloned();
                Ok(Self::RemovedPrefab { template })
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
            (Self::DeleteNode { node, components }, EditorCommandResult::None) => {
                Ok(Self::DeleteNode { node, components })
            }
            (
                Self::DuplicateNode {
                    source,
                    node: None,
                    components: None,
                },
                EditorCommandResult::CreatedNode(entity),
            ) => {
                let node = engine
                    .active_scene
                    .node(*entity)
                    .ok_or(SceneError::EntityNotFound)?
                    .clone();
                let components = NodeComponentSnapshot::capture(engine, *entity)?;
                Ok(Self::DuplicateNode {
                    source,
                    node: Some(node),
                    components: Some(components),
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
                Self::AttachCamera {
                    entity,
                    before,
                    after,
                },
                EditorCommandResult::None,
            ) => Ok(Self::AttachCamera {
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
                Self::RemoveComponent {
                    entity,
                    before,
                    after: None,
                },
                EditorCommandResult::None,
            ) => Ok(Self::RemoveComponent {
                entity,
                before,
                after: Some(NodeComponentSnapshot::capture(engine, entity)?),
            }),
            (
                Self::ApplyGameplayPreset {
                    entity,
                    before,
                    after: None,
                },
                EditorCommandResult::None,
            ) => Ok(Self::ApplyGameplayPreset {
                entity,
                before,
                after: Some(NodeComponentSnapshot::capture(engine, entity)?),
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
            (
                Self::SpawnedNode {
                    node: None,
                    components: None,
                },
                EditorCommandResult::CreatedNode(entity),
            ) => {
                let node = engine
                    .active_scene
                    .node(*entity)
                    .ok_or(SceneError::EntityNotFound)?
                    .clone();
                let components = NodeComponentSnapshot::capture(engine, *entity)?;
                Ok(Self::SpawnedNode {
                    node: Some(node),
                    components: Some(components),
                })
            }
            (
                Self::RegisteredPrefab {
                    prefab_name,
                    previous,
                    registered: None,
                },
                EditorCommandResult::None,
            ) => {
                let registered = engine
                    .prefabs
                    .get(&prefab_name)
                    .cloned()
                    .ok_or(CommandHistoryError::UnexpectedCommandResult)?;
                Ok(Self::RegisteredPrefab {
                    prefab_name,
                    previous,
                    registered: Some(registered),
                })
            }
            (Self::RemovedPrefab { template }, EditorCommandResult::None) => {
                Ok(Self::RemovedPrefab { template })
            }
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
            Self::DeleteNode { node, components } => {
                engine.active_scene.restore_node(node.clone())?;
                components.apply_to_entity(engine, node.id)?;
                Ok(())
            }
            Self::DuplicateNode {
                node: Some(node), ..
            } => {
                engine.active_scene.despawn_node(node.id)?;
                Ok(())
            }
            Self::DuplicateNode { node: None, .. } => {
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
            Self::AttachCamera { entity, before, .. } => {
                if let Some(component) = before {
                    engine.active_scene.add_camera(*entity, *component)?;
                } else {
                    engine.active_scene.remove_camera(*entity)?;
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
            Self::RemoveComponent { entity, before, .. }
            | Self::ApplyGameplayPreset { entity, before, .. } => {
                before.apply_to_entity(engine, *entity)?;
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
            Self::SpawnedNode {
                node: Some(node), ..
            } => {
                engine.active_scene.despawn_node(node.id)?;
                Ok(())
            }
            Self::SpawnedNode { node: None, .. } => {
                Err(CommandHistoryError::UnexpectedCommandResult)
            }
            Self::RegisteredPrefab {
                prefab_name,
                previous,
                registered: _,
            } => {
                engine.prefabs.remove(prefab_name);
                if let Some(prev) = previous {
                    engine.prefabs.register(prev.clone());
                }
                Ok(())
            }
            Self::RemovedPrefab {
                template: Some(template),
            } => {
                engine.prefabs.register(template.clone());
                Ok(())
            }
            Self::RemovedPrefab { template: None } => Ok(()),
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
            Self::DeleteNode { node, components } => {
                engine.active_scene.despawn_node(node.id)?;
                Ok(Self::DeleteNode { node, components })
            }
            Self::DuplicateNode {
                source,
                node: Some(node),
                components: Some(components),
            } => {
                engine.active_scene.restore_node(node.clone())?;
                components.apply_to_entity(engine, node.id)?;
                Ok(Self::DuplicateNode {
                    source,
                    node: Some(node),
                    components: Some(components),
                })
            }
            Self::DuplicateNode { .. } => Err(CommandHistoryError::UnexpectedCommandResult),
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
            Self::AttachCamera {
                entity,
                before,
                after,
            } => {
                engine.active_scene.add_camera(entity, after)?;
                Ok(Self::AttachCamera {
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
            Self::RemoveComponent {
                entity,
                before,
                after: Some(after),
            } => {
                after.apply_to_entity(engine, entity)?;
                Ok(Self::RemoveComponent {
                    entity,
                    before,
                    after: Some(after),
                })
            }
            Self::ApplyGameplayPreset {
                entity,
                before,
                after: Some(after),
            } => {
                after.apply_to_entity(engine, entity)?;
                Ok(Self::ApplyGameplayPreset {
                    entity,
                    before,
                    after: Some(after),
                })
            }
            Self::RemoveComponent { .. } | Self::ApplyGameplayPreset { .. } => {
                Err(CommandHistoryError::UnexpectedCommandResult)
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
            Self::SpawnedNode {
                node: Some(ref node),
                components: Some(ref components),
            } => {
                engine.active_scene.restore_node(node.clone())?;
                components.apply_to_entity(engine, node.id)?;
                Ok(self)
            }
            Self::SpawnedNode { .. } => Err(CommandHistoryError::UnexpectedCommandResult),
            Self::RegisteredPrefab {
                prefab_name,
                previous,
                registered: Some(registered),
            } => {
                engine.prefabs.register(registered.clone());
                Ok(Self::RegisteredPrefab {
                    prefab_name,
                    previous,
                    registered: Some(registered),
                })
            }
            Self::RegisteredPrefab {
                registered: None, ..
            } => Err(CommandHistoryError::UnexpectedCommandResult),
            Self::RemovedPrefab { template } => {
                if let Some(ref t) = template {
                    engine.prefabs.remove(&t.name);
                }
                Ok(Self::RemovedPrefab { template })
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandHistoryError {
    Command(EditorCommandError),
    MissingComponent,
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
            Self::MissingComponent => formatter.write_str("component was not found"),
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
            Self::MissingComponent
            | Self::MissingTilemap
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

fn snapshot_has_component(
    snapshot: &NodeComponentSnapshot,
    component: EditorComponentKind,
) -> bool {
    match component {
        EditorComponentKind::Tag => snapshot.tag.is_some(),
        EditorComponentKind::Sprite => snapshot.sprite.is_some(),
        EditorComponentKind::Camera => snapshot.camera.is_some(),
        EditorComponentKind::Tilemap => snapshot.tilemap.is_some(),
        EditorComponentKind::Velocity => snapshot.velocity.is_some(),
        EditorComponentKind::Collider => snapshot.collider.is_some(),
        EditorComponentKind::PlayerController => snapshot.player_controller.is_some(),
        EditorComponentKind::CameraFollow => snapshot.camera_follow.is_some(),
        EditorComponentKind::Trigger => snapshot.trigger.is_some(),
        EditorComponentKind::Behavior => snapshot.behavior.is_some(),
        EditorComponentKind::Audio => snapshot.audio.is_some(),
        EditorComponentKind::Animation => snapshot.animation.is_some(),
        EditorComponentKind::UiLabel => snapshot.ui_label.is_some(),
        EditorComponentKind::UiPanel => snapshot.ui_panel.is_some(),
        EditorComponentKind::ParticleEmitter => snapshot.particle_emitter.is_some(),
    }
}

#[cfg(test)]
mod tests {
    use crab2d_core::{Engine, EngineConfig};
    use crab2d_scene::{
        AnimationComponent, AudioComponent, BehaviorComponent, Collider2DComponent,
        ParticleEmitterComponent, SpriteComponent, TileCell, TileSize, TilemapComponent,
        TilemapSize, UiLabelComponent, UiPanelComponent, Vec2,
    };

    use crate::{CommandHistory, EditorCommand, EditorComponentKind};

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

    #[test]
    fn remove_component_can_be_undone_and_redone() {
        let mut engine = test_engine();
        let mut history = CommandHistory::default();
        let wall = engine.active_scene.spawn_node("Wall");
        engine
            .active_scene
            .add_collider(wall, Collider2DComponent::rectangle(Vec2::new(32.0, 32.0)))
            .expect("collider should attach");

        history
            .execute(
                EditorCommand::remove_component(wall, EditorComponentKind::Collider),
                &mut engine,
            )
            .expect("remove should execute");
        assert!(engine.active_scene.collider(wall).is_none());

        history.undo(&mut engine).expect("undo should succeed");
        assert!(engine.active_scene.collider(wall).is_some());

        history.redo(&mut engine).expect("redo should succeed");
        assert!(engine.active_scene.collider(wall).is_none());
    }

    #[test]
    fn remove_extended_components_can_be_undone_and_redone() {
        let cases = [
            EditorComponentKind::Behavior,
            EditorComponentKind::Audio,
            EditorComponentKind::Animation,
            EditorComponentKind::UiLabel,
            EditorComponentKind::UiPanel,
            EditorComponentKind::ParticleEmitter,
        ];

        for component in cases {
            let mut engine = test_engine();
            let mut history = CommandHistory::default();
            let entity = engine.active_scene.spawn_node("Entity");
            attach_component(&mut engine, entity, component);
            assert_component_state(&engine, entity, component, true);

            history
                .execute(
                    EditorCommand::remove_component(entity, component),
                    &mut engine,
                )
                .expect("remove should execute");
            assert_component_state(&engine, entity, component, false);

            history.undo(&mut engine).expect("undo should succeed");
            assert_component_state(&engine, entity, component, true);

            history.redo(&mut engine).expect("redo should succeed");
            assert_component_state(&engine, entity, component, false);
        }
    }

    #[test]
    fn create_prefab_can_be_undone_and_redone() {
        let mut engine = test_engine();
        let mut history = CommandHistory::default();
        let source = engine.active_scene.spawn_node("Hero");
        engine
            .active_scene
            .add_sprite(source, SpriteComponent::new("sprites/hero.png"))
            .expect("sprite should attach");

        history
            .execute(
                EditorCommand::create_prefab_from_entity(source, "HeroPrefab"),
                &mut engine,
            )
            .expect("create prefab should execute");
        assert!(engine.prefabs.get("HeroPrefab").is_some());

        history.undo(&mut engine).expect("undo should succeed");
        assert!(engine.prefabs.get("HeroPrefab").is_none());

        history.redo(&mut engine).expect("redo should succeed");
        assert_eq!(
            engine
                .prefabs
                .get("HeroPrefab")
                .and_then(|prefab| prefab.sprite.as_ref())
                .map(|sprite| sprite.sprite_path.as_str()),
            Some("sprites/hero.png")
        );
    }

    #[test]
    fn remove_prefab_can_be_undone_and_redone() {
        let mut engine = test_engine();
        let mut history = CommandHistory::default();
        let source = engine.active_scene.spawn_node("Coin");
        engine
            .active_scene
            .add_sprite(source, SpriteComponent::new("sprites/coin.png"))
            .expect("sprite should attach");
        EditorCommand::create_prefab_from_entity(source, "CoinPrefab")
            .apply(&mut engine)
            .expect("prefab should register");

        history
            .execute(EditorCommand::remove_prefab("CoinPrefab"), &mut engine)
            .expect("remove prefab should execute");
        assert!(engine.prefabs.get("CoinPrefab").is_none());

        history.undo(&mut engine).expect("undo should succeed");
        assert!(engine.prefabs.get("CoinPrefab").is_some());

        history.redo(&mut engine).expect("redo should succeed");
        assert!(engine.prefabs.get("CoinPrefab").is_none());
    }

    fn test_engine() -> Engine {
        Engine::new(EngineConfig::new("Crab2D Test"))
    }

    fn test_tilemap() -> TilemapComponent {
        TilemapComponent::new(TilemapSize::new(4, 4), TileSize::new(32, 32))
            .expect("tilemap should be valid")
    }

    fn attach_component(
        engine: &mut Engine,
        entity: crab2d_scene::EntityId,
        component: EditorComponentKind,
    ) {
        match component {
            EditorComponentKind::Behavior => engine
                .active_scene
                .add_behavior(entity, BehaviorComponent::new("scripts/entity.rhai"))
                .expect("behavior should attach"),
            EditorComponentKind::Audio => engine
                .active_scene
                .add_audio(entity, AudioComponent::new("audio/sound.wav"))
                .expect("audio should attach"),
            EditorComponentKind::Animation => engine
                .active_scene
                .add_animation(
                    entity,
                    AnimationComponent::new("sprites/sheet.png", 16, 16, 4),
                )
                .expect("animation should attach"),
            EditorComponentKind::UiLabel => engine
                .active_scene
                .add_ui_label(entity, UiLabelComponent::new("Score: 0"))
                .expect("ui label should attach"),
            EditorComponentKind::UiPanel => engine
                .active_scene
                .add_ui_panel(entity, UiPanelComponent::new(120.0, 32.0))
                .expect("ui panel should attach"),
            EditorComponentKind::ParticleEmitter => engine
                .active_scene
                .add_particle_emitter(entity, ParticleEmitterComponent::new("sprites/spark.png"))
                .expect("particle emitter should attach"),
            _ => panic!("test helper only supports extended components"),
        }
    }

    fn assert_component_state(
        engine: &Engine,
        entity: crab2d_scene::EntityId,
        component: EditorComponentKind,
        expected: bool,
    ) {
        let exists = match component {
            EditorComponentKind::Behavior => engine.active_scene.behavior(entity).is_some(),
            EditorComponentKind::Audio => engine.active_scene.audio(entity).is_some(),
            EditorComponentKind::Animation => engine.active_scene.animation(entity).is_some(),
            EditorComponentKind::UiLabel => engine.active_scene.ui_label(entity).is_some(),
            EditorComponentKind::UiPanel => engine.active_scene.ui_panel(entity).is_some(),
            EditorComponentKind::ParticleEmitter => {
                engine.active_scene.particle_emitter(entity).is_some()
            }
            _ => panic!("test helper only supports extended components"),
        };
        assert_eq!(
            exists, expected,
            "component state mismatch for {component:?}"
        );
    }
}
