use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

use crab2d_core::Engine;
use crab2d_scene::{
    Camera2DComponent, CameraFollowComponent, Collider2DComponent, EntityId,
    PlayerControllerComponent, SceneError, SpriteComponent, TagComponent, TileCell, TileSize,
    TilemapComponent, TilemapError, TilemapSize, Transform2D, TriggerComponent, Vec2,
    Velocity2DComponent,
};

#[derive(Debug, Clone, PartialEq)]
pub enum EditorCommand {
    CreateNode {
        name: String,
    },
    DeleteNode {
        entity: EntityId,
    },
    DuplicateNode {
        entity: EntityId,
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
    AttachCamera {
        entity: EntityId,
        camera: Camera2DComponent,
    },
    AttachTilemap {
        entity: EntityId,
        tilemap: TilemapComponent,
    },
    AttachVelocity {
        entity: EntityId,
        velocity: Velocity2DComponent,
    },
    AttachCollider {
        entity: EntityId,
        collider: Collider2DComponent,
    },
    AttachPlayerController {
        entity: EntityId,
        controller: PlayerControllerComponent,
    },
    AttachCameraFollow {
        entity: EntityId,
        follow: CameraFollowComponent,
    },
    AttachTrigger {
        entity: EntityId,
        trigger: TriggerComponent,
    },
    RemoveComponent {
        entity: EntityId,
        component: EditorComponentKind,
    },
    ApplyGameplayPreset {
        entity: EntityId,
        preset: GameplayPreset,
    },
    SetTileCollision {
        entity: EntityId,
        solid_tiles: BTreeSet<u32>,
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

    pub fn delete_node(entity: EntityId) -> Self {
        Self::DeleteNode { entity }
    }

    pub fn duplicate_node(entity: EntityId) -> Self {
        Self::DuplicateNode { entity }
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

    pub fn attach_camera(entity: EntityId, camera: Camera2DComponent) -> Self {
        Self::AttachCamera { entity, camera }
    }

    pub fn attach_tilemap(entity: EntityId, tilemap: TilemapComponent) -> Self {
        Self::AttachTilemap { entity, tilemap }
    }

    pub fn attach_velocity(entity: EntityId, velocity: Velocity2DComponent) -> Self {
        Self::AttachVelocity { entity, velocity }
    }

    pub fn attach_collider(entity: EntityId, collider: Collider2DComponent) -> Self {
        Self::AttachCollider { entity, collider }
    }

    pub fn attach_player_controller(
        entity: EntityId,
        controller: PlayerControllerComponent,
    ) -> Self {
        Self::AttachPlayerController { entity, controller }
    }

    pub fn attach_camera_follow(entity: EntityId, follow: CameraFollowComponent) -> Self {
        Self::AttachCameraFollow { entity, follow }
    }

    pub fn attach_trigger(entity: EntityId, trigger: TriggerComponent) -> Self {
        Self::AttachTrigger { entity, trigger }
    }

    pub fn remove_component(entity: EntityId, component: EditorComponentKind) -> Self {
        Self::RemoveComponent { entity, component }
    }

    pub fn apply_gameplay_preset(entity: EntityId, preset: GameplayPreset) -> Self {
        Self::ApplyGameplayPreset { entity, preset }
    }

    pub fn set_tile_collision(entity: EntityId, solid_tiles: BTreeSet<u32>) -> Self {
        Self::SetTileCollision {
            entity,
            solid_tiles,
        }
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
            Self::DeleteNode { entity } => {
                engine.active_scene.despawn_node(entity)?;
                Ok(EditorCommandResult::None)
            }
            Self::DuplicateNode { entity } => {
                let source = engine
                    .active_scene
                    .node(entity)
                    .ok_or(SceneError::EntityNotFound)?
                    .clone();
                let snapshot = NodeComponentSnapshot::capture(engine, entity)?;
                let mut transform = source.transform;
                transform.position += Vec2::new(24.0, -24.0);
                let duplicate = engine
                    .active_scene
                    .spawn_node_with_transform(next_copy_name(&source.name), transform)?;
                snapshot.apply_to_entity(engine, duplicate)?;
                Ok(EditorCommandResult::CreatedNode(duplicate))
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
            Self::AttachCamera { entity, camera } => {
                engine.active_scene.add_camera(entity, camera)?;
                Ok(EditorCommandResult::None)
            }
            Self::AttachTilemap { entity, tilemap } => {
                engine.active_scene.add_tilemap(entity, tilemap)?;
                Ok(EditorCommandResult::None)
            }
            Self::AttachVelocity { entity, velocity } => {
                engine.active_scene.add_velocity(entity, velocity)?;
                Ok(EditorCommandResult::None)
            }
            Self::AttachCollider { entity, collider } => {
                engine.active_scene.add_collider(entity, collider)?;
                Ok(EditorCommandResult::None)
            }
            Self::AttachPlayerController { entity, controller } => {
                engine
                    .active_scene
                    .add_player_controller(entity, controller)?;
                Ok(EditorCommandResult::None)
            }
            Self::AttachCameraFollow { entity, follow } => {
                engine.active_scene.add_camera_follow(entity, follow)?;
                Ok(EditorCommandResult::None)
            }
            Self::AttachTrigger { entity, trigger } => {
                engine.active_scene.add_trigger(entity, trigger)?;
                Ok(EditorCommandResult::None)
            }
            Self::RemoveComponent { entity, component } => {
                remove_component(engine, entity, component)?;
                Ok(EditorCommandResult::None)
            }
            Self::ApplyGameplayPreset { entity, preset } => {
                apply_gameplay_preset(engine, entity, preset)?;
                Ok(EditorCommandResult::None)
            }
            Self::SetTileCollision {
                entity,
                solid_tiles,
            } => {
                let tilemap = engine
                    .active_scene
                    .tilemap_mut(entity)
                    .ok_or(EditorCommandError::MissingTilemap)?;
                tilemap.collision.solid_tiles = solid_tiles;
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
pub enum EditorComponentKind {
    Tag,
    Sprite,
    Camera,
    Tilemap,
    Velocity,
    Collider,
    PlayerController,
    CameraFollow,
    Trigger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameplayPreset {
    TopDownPlayer,
    StaticWall,
    Collectible,
    Door,
    TriggerArea,
    CameraFollow,
}

impl GameplayPreset {
    pub const ALL: [Self; 6] = [
        Self::TopDownPlayer,
        Self::StaticWall,
        Self::Collectible,
        Self::Door,
        Self::TriggerArea,
        Self::CameraFollow,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::TopDownPlayer => "Top Down Player",
            Self::StaticWall => "Static Wall",
            Self::Collectible => "Collectible",
            Self::Door => "Door",
            Self::TriggerArea => "Trigger Area",
            Self::CameraFollow => "Camera Follow",
        }
    }

    pub const fn default_node_name(self) -> &'static str {
        match self {
            Self::TopDownPlayer => "Player",
            Self::StaticWall => "Wall",
            Self::Collectible => "Collectible",
            Self::Door => "Door",
            Self::TriggerArea => "TriggerArea",
            Self::CameraFollow => "Camera2D",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NodeComponentSnapshot {
    pub tag: Option<TagComponent>,
    pub sprite: Option<SpriteComponent>,
    pub camera: Option<Camera2DComponent>,
    pub tilemap: Option<TilemapComponent>,
    pub velocity: Option<Velocity2DComponent>,
    pub collider: Option<Collider2DComponent>,
    pub player_controller: Option<PlayerControllerComponent>,
    pub camera_follow: Option<CameraFollowComponent>,
    pub trigger: Option<TriggerComponent>,
}

impl NodeComponentSnapshot {
    pub fn capture(engine: &Engine, entity: EntityId) -> Result<Self, SceneError> {
        engine
            .active_scene
            .node(entity)
            .ok_or(SceneError::EntityNotFound)?;

        Ok(Self {
            tag: engine.active_scene.tag(entity).cloned(),
            sprite: engine.active_scene.sprite(entity).cloned(),
            camera: engine.active_scene.camera(entity).copied(),
            tilemap: engine.active_scene.tilemap(entity).cloned(),
            velocity: engine.active_scene.velocity(entity).copied(),
            collider: engine.active_scene.collider(entity).copied(),
            player_controller: engine.active_scene.player_controller(entity).copied(),
            camera_follow: engine.active_scene.camera_follow(entity).copied(),
            trigger: engine.active_scene.trigger(entity).cloned(),
        })
    }

    pub fn apply_to_entity(&self, engine: &mut Engine, entity: EntityId) -> Result<(), SceneError> {
        if let Some(component) = &self.tag {
            engine.active_scene.add_tag(entity, component.clone())?;
        } else {
            engine.active_scene.remove_tag(entity)?;
        }

        if let Some(component) = &self.sprite {
            engine.active_scene.add_sprite(entity, component.clone())?;
        } else {
            engine.active_scene.remove_sprite(entity)?;
        }

        if let Some(component) = self.camera {
            engine.active_scene.add_camera(entity, component)?;
        } else {
            engine.active_scene.remove_camera(entity)?;
        }

        if let Some(component) = &self.tilemap {
            engine.active_scene.add_tilemap(entity, component.clone())?;
        } else {
            engine.active_scene.remove_tilemap(entity)?;
        }

        if let Some(component) = self.velocity {
            engine.active_scene.add_velocity(entity, component)?;
        } else {
            engine.active_scene.remove_velocity(entity)?;
        }

        if let Some(component) = self.collider {
            engine.active_scene.add_collider(entity, component)?;
        } else {
            engine.active_scene.remove_collider(entity)?;
        }

        if let Some(component) = self.player_controller {
            engine
                .active_scene
                .add_player_controller(entity, component)?;
        } else {
            engine.active_scene.remove_player_controller(entity)?;
        }

        if let Some(component) = self.camera_follow {
            engine.active_scene.add_camera_follow(entity, component)?;
        } else {
            engine.active_scene.remove_camera_follow(entity)?;
        }

        if let Some(component) = &self.trigger {
            engine.active_scene.add_trigger(entity, component.clone())?;
        } else {
            engine.active_scene.remove_trigger(entity)?;
        }

        Ok(())
    }
}

fn remove_component(
    engine: &mut Engine,
    entity: EntityId,
    component: EditorComponentKind,
) -> Result<(), SceneError> {
    match component {
        EditorComponentKind::Tag => {
            engine.active_scene.remove_tag(entity)?;
        }
        EditorComponentKind::Sprite => {
            engine.active_scene.remove_sprite(entity)?;
        }
        EditorComponentKind::Camera => {
            engine.active_scene.remove_camera(entity)?;
        }
        EditorComponentKind::Tilemap => {
            engine.active_scene.remove_tilemap(entity)?;
        }
        EditorComponentKind::Velocity => {
            engine.active_scene.remove_velocity(entity)?;
        }
        EditorComponentKind::Collider => {
            engine.active_scene.remove_collider(entity)?;
        }
        EditorComponentKind::PlayerController => {
            engine.active_scene.remove_player_controller(entity)?;
        }
        EditorComponentKind::CameraFollow => {
            engine.active_scene.remove_camera_follow(entity)?;
        }
        EditorComponentKind::Trigger => {
            engine.active_scene.remove_trigger(entity)?;
        }
    }
    Ok(())
}

fn apply_gameplay_preset(
    engine: &mut Engine,
    entity: EntityId,
    preset: GameplayPreset,
) -> Result<(), SceneError> {
    match preset {
        GameplayPreset::TopDownPlayer => {
            engine
                .active_scene
                .add_tag(entity, TagComponent::new("player"))?;
            engine
                .active_scene
                .add_sprite(entity, SpriteComponent::new("sprites/player.png"))?;
            engine
                .active_scene
                .add_velocity(entity, Velocity2DComponent::default())?;
            engine.active_scene.add_collider(
                entity,
                Collider2DComponent::rectangle(Vec2::new(24.0, 24.0)),
            )?;
            engine
                .active_scene
                .add_player_controller(entity, PlayerControllerComponent::default())?;
        }
        GameplayPreset::StaticWall => {
            engine
                .active_scene
                .add_tag(entity, TagComponent::new("wall"))?;
            engine.active_scene.add_collider(
                entity,
                Collider2DComponent::rectangle(Vec2::new(64.0, 64.0)),
            )?;
        }
        GameplayPreset::Collectible => {
            engine
                .active_scene
                .add_tag(entity, TagComponent::new("collectible"))?;
            engine.active_scene.add_collider(
                entity,
                Collider2DComponent::rectangle(Vec2::new(24.0, 24.0)).sensor(),
            )?;
            engine
                .active_scene
                .add_trigger(entity, TriggerComponent::new("collectible").once())?;
        }
        GameplayPreset::Door => {
            engine
                .active_scene
                .add_tag(entity, TagComponent::new("door"))?;
            engine.active_scene.add_collider(
                entity,
                Collider2DComponent::rectangle(Vec2::new(32.0, 48.0)).sensor(),
            )?;
            engine
                .active_scene
                .add_trigger(entity, TriggerComponent::new("door"))?;
        }
        GameplayPreset::TriggerArea => {
            engine
                .active_scene
                .add_tag(entity, TagComponent::new("trigger"))?;
            engine.active_scene.add_collider(
                entity,
                Collider2DComponent::rectangle(Vec2::new(64.0, 64.0)).sensor(),
            )?;
            engine
                .active_scene
                .add_trigger(entity, TriggerComponent::new("trigger"))?;
        }
        GameplayPreset::CameraFollow => {
            engine
                .active_scene
                .add_camera(entity, Camera2DComponent::default())?;
            let target = engine
                .active_scene
                .find_node_by_tag("player")
                .map(|node| node.id)
                .unwrap_or(entity);
            engine
                .active_scene
                .add_camera_follow(entity, CameraFollowComponent::new(target))?;
        }
    }
    Ok(())
}

pub fn default_tilemap() -> Result<TilemapComponent, TilemapError> {
    let mut tilemap = TilemapComponent::new(TilemapSize::new(12, 8), TileSize::new(32, 32))?;
    tilemap.collision.set_solid(3, true);
    Ok(tilemap)
}

fn next_copy_name(name: &str) -> String {
    format!("{name} Copy")
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

    use crate::{EditorCommand, EditorCommandError, EditorCommandResult, GameplayPreset};

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

    #[test]
    fn gameplay_preset_adds_player_runtime_components() {
        let mut engine = test_engine();
        let player = engine.active_scene.spawn_node("Player");

        EditorCommand::apply_gameplay_preset(player, GameplayPreset::TopDownPlayer)
            .apply(&mut engine)
            .expect("preset should apply");

        assert!(engine.active_scene.sprite(player).is_some());
        assert!(engine.active_scene.collider(player).is_some());
        assert!(engine.active_scene.velocity(player).is_some());
        assert!(engine.active_scene.player_controller(player).is_some());
    }

    fn test_engine() -> Engine {
        Engine::new(EngineConfig::new("Crab2D Test"))
    }

    fn test_tilemap() -> TilemapComponent {
        TilemapComponent::new(TilemapSize::new(4, 4), TileSize::new(32, 32))
            .expect("tilemap should be valid")
    }
}
