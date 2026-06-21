use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

use crab2d_core::Engine;
use crab2d_scene::{
    Camera2DComponent, CameraFollowComponent, Collider2DComponent, EntityId,
    PlayerControllerComponent, PrefabTemplate, SceneError, SpriteComponent, TagComponent, TileCell,
    TileSize, TilemapComponent, TilemapError, TilemapSize, Transform2D, TriggerComponent, UiAnchor,
    UiLabelComponent, Vec2, Velocity2DComponent, WorldTextComponent,
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
    CreateFromAsset {
        name: String,
        sprite_path: String,
        transform: Transform2D,
    },
    CreateCamera {
        name: String,
    },
    CreateWorldTextNode {
        name: String,
        text: String,
    },
    CreateScenePortal {
        name: String,
        target_scene: String,
    },
    CreatePrefabFromEntity {
        entity: EntityId,
        prefab_name: String,
    },
    InstantiatePrefab {
        prefab_name: String,
        transform: Transform2D,
    },
    RemovePrefab {
        prefab_name: String,
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

    pub fn create_from_asset(name: impl Into<String>, sprite_path: impl Into<String>) -> Self {
        Self::create_from_asset_at(name, sprite_path, Transform2D::IDENTITY)
    }

    pub fn create_from_asset_at(
        name: impl Into<String>,
        sprite_path: impl Into<String>,
        transform: Transform2D,
    ) -> Self {
        Self::CreateFromAsset {
            name: name.into(),
            sprite_path: sprite_path.into(),
            transform,
        }
    }

    pub fn create_camera(name: impl Into<String>) -> Self {
        Self::CreateCamera { name: name.into() }
    }

    pub fn create_world_text_node(name: impl Into<String>, text: impl Into<String>) -> Self {
        Self::CreateWorldTextNode {
            name: name.into(),
            text: text.into(),
        }
    }

    pub fn create_scene_portal(name: impl Into<String>, target_scene: impl Into<String>) -> Self {
        Self::CreateScenePortal {
            name: name.into(),
            target_scene: target_scene.into(),
        }
    }

    pub fn create_prefab_from_entity(entity: EntityId, prefab_name: impl Into<String>) -> Self {
        Self::CreatePrefabFromEntity {
            entity,
            prefab_name: prefab_name.into(),
        }
    }

    pub fn instantiate_prefab(prefab_name: impl Into<String>, transform: Transform2D) -> Self {
        Self::InstantiatePrefab {
            prefab_name: prefab_name.into(),
            transform,
        }
    }

    pub fn remove_prefab(prefab_name: impl Into<String>) -> Self {
        Self::RemovePrefab {
            prefab_name: prefab_name.into(),
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
            Self::CreateFromAsset {
                name,
                sprite_path,
                transform,
            } => {
                let entity = engine
                    .active_scene
                    .spawn_node_with_transform(name, transform)?;
                engine
                    .active_scene
                    .add_sprite(entity, SpriteComponent::new(sprite_path))?;
                Ok(EditorCommandResult::CreatedNode(entity))
            }
            Self::CreateCamera { name } => {
                let entity = engine.active_scene.try_spawn_node(name)?;
                engine
                    .active_scene
                    .add_camera(entity, Camera2DComponent::default())?;
                Ok(EditorCommandResult::CreatedNode(entity))
            }
            Self::CreateWorldTextNode { name, text } => {
                let entity = engine.active_scene.try_spawn_node(name)?;
                engine
                    .active_scene
                    .add_world_text(entity, WorldTextComponent::new(text))?;
                Ok(EditorCommandResult::CreatedNode(entity))
            }
            Self::CreateScenePortal { name, target_scene } => {
                let entity = engine.active_scene.try_spawn_node(name)?;
                engine
                    .active_scene
                    .add_tag(entity, TagComponent::new("portal"))?;
                engine.active_scene.add_collider(
                    entity,
                    Collider2DComponent::rectangle(Vec2::new(32.0, 48.0)).sensor(),
                )?;
                engine.active_scene.add_trigger(
                    entity,
                    TriggerComponent::new(format!("scene:{target_scene}")),
                )?;
                Ok(EditorCommandResult::CreatedNode(entity))
            }
            Self::CreatePrefabFromEntity {
                entity,
                prefab_name,
            } => {
                let template = PrefabTemplate::from_entity(&engine.active_scene, entity)
                    .ok_or(SceneError::EntityNotFound)?;
                let named = PrefabTemplate {
                    name: prefab_name,
                    ..template
                };
                engine.prefabs.register(named);
                Ok(EditorCommandResult::None)
            }
            Self::InstantiatePrefab {
                prefab_name,
                transform,
            } => {
                let template = engine
                    .prefabs
                    .get(&prefab_name)
                    .ok_or(EditorCommandError::PrefabNotFound)?
                    .clone();
                let entity = template.instantiate(&mut engine.active_scene, transform)?;
                Ok(EditorCommandResult::CreatedNode(entity))
            }
            Self::RemovePrefab { prefab_name } => {
                engine.prefabs.remove(&prefab_name);
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
    Behavior,
    Audio,
    Animation,
    UiLabel,
    UiPanel,
    ParticleEmitter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameplayPreset {
    TopDownPlayer,
    StaticWall,
    Collectible,
    Door,
    TriggerArea,
    CameraFollow,
    DamageZone,
    Checkpoint,
    WorldSign,
    HudLabel,
}

impl GameplayPreset {
    pub const ALL: [Self; 10] = [
        Self::TopDownPlayer,
        Self::StaticWall,
        Self::Collectible,
        Self::Door,
        Self::TriggerArea,
        Self::CameraFollow,
        Self::DamageZone,
        Self::Checkpoint,
        Self::WorldSign,
        Self::HudLabel,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::TopDownPlayer => "Top Down Player",
            Self::StaticWall => "Static Wall",
            Self::Collectible => "Collectible",
            Self::Door => "Door",
            Self::TriggerArea => "Trigger Area",
            Self::CameraFollow => "Camera Follow",
            Self::DamageZone => "Damage Zone",
            Self::Checkpoint => "Checkpoint",
            Self::WorldSign => "World Sign",
            Self::HudLabel => "HUD Label",
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
            Self::DamageZone => "DamageZone",
            Self::Checkpoint => "Checkpoint",
            Self::WorldSign => "Sign",
            Self::HudLabel => "HudLabel",
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
    pub world_text: Option<WorldTextComponent>,
    pub ui_label: Option<UiLabelComponent>,
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
            world_text: engine.active_scene.world_text(entity).cloned(),
            ui_label: engine.active_scene.ui_label(entity).cloned(),
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

        if let Some(component) = &self.world_text {
            engine
                .active_scene
                .add_world_text(entity, component.clone())?;
        } else {
            engine.active_scene.remove_world_text(entity);
        }

        if let Some(component) = &self.ui_label {
            engine
                .active_scene
                .add_ui_label(entity, component.clone())?;
        } else {
            engine.active_scene.remove_ui_label(entity);
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
        EditorComponentKind::Behavior => {
            engine.active_scene.remove_behavior(entity);
        }
        EditorComponentKind::Audio => {
            engine.active_scene.remove_audio(entity);
        }
        EditorComponentKind::Animation => {
            engine.active_scene.remove_animation(entity);
        }
        EditorComponentKind::UiLabel => {
            engine.active_scene.remove_ui_label(entity);
        }
        EditorComponentKind::UiPanel => {
            engine.active_scene.remove_ui_panel(entity);
        }
        EditorComponentKind::ParticleEmitter => {
            engine.active_scene.remove_particle_emitter(entity);
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
        GameplayPreset::DamageZone => {
            engine
                .active_scene
                .add_tag(entity, TagComponent::new("damage"))?;
            engine.active_scene.add_collider(
                entity,
                Collider2DComponent::rectangle(Vec2::new(64.0, 64.0)).sensor(),
            )?;
            engine
                .active_scene
                .add_trigger(entity, TriggerComponent::new("damage"))?;
        }
        GameplayPreset::Checkpoint => {
            engine
                .active_scene
                .add_tag(entity, TagComponent::new("checkpoint"))?;
            engine.active_scene.add_collider(
                entity,
                Collider2DComponent::rectangle(Vec2::new(48.0, 64.0)).sensor(),
            )?;
            engine
                .active_scene
                .add_trigger(entity, TriggerComponent::new("checkpoint").once())?;
        }
        GameplayPreset::WorldSign => {
            engine
                .active_scene
                .add_tag(entity, TagComponent::new("sign"))?;
            engine
                .active_scene
                .add_world_text(entity, WorldTextComponent::new("..."))?;
        }
        GameplayPreset::HudLabel => {
            engine.active_scene.add_ui_label(
                entity,
                UiLabelComponent::new("Score: 0")
                    .with_anchor(UiAnchor::TopLeft)
                    .at(8.0, 8.0),
            )?;
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
    PrefabNotFound,
    Scene(SceneError),
    Tilemap(TilemapError),
}

impl fmt::Display for EditorCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingTilemap => formatter.write_str("tilemap component was not found"),
            Self::PrefabNotFound => formatter.write_str("prefab was not found in the registry"),
            Self::Scene(error) => write!(formatter, "{error}"),
            Self::Tilemap(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for EditorCommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MissingTilemap => None,
            Self::PrefabNotFound => None,
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
    use crab2d_scene::{
        SceneError, TileCell, TileSize, TilemapComponent, TilemapSize, Transform2D, Vec2,
    };

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

    #[test]
    fn gameplay_preset_damage_zone_adds_tag_collider_and_trigger() {
        let mut engine = test_engine();
        let zone = engine.active_scene.spawn_node("DamageZone");

        EditorCommand::apply_gameplay_preset(zone, GameplayPreset::DamageZone)
            .apply(&mut engine)
            .expect("damage zone preset should apply");

        assert_eq!(
            engine.active_scene.tag(zone).expect("tag exists").tag,
            "damage"
        );
        let collider = engine.active_scene.collider(zone).expect("collider exists");
        assert!(collider.is_sensor);
        let trigger = engine.active_scene.trigger(zone).expect("trigger exists");
        assert_eq!(trigger.name, "damage");
    }

    #[test]
    fn gameplay_preset_hud_label_adds_ui_label() {
        let mut engine = test_engine();
        let hud = engine.active_scene.spawn_node("ScoreLabel");

        EditorCommand::apply_gameplay_preset(hud, GameplayPreset::HudLabel)
            .apply(&mut engine)
            .expect("hud label preset should apply");

        let label = engine
            .active_scene
            .ui_label(hud)
            .expect("ui_label should exist");
        assert_eq!(label.text, "Score: 0");
        assert_eq!(label.offset_x, 8.0);
        assert_eq!(label.offset_y, 8.0);
    }

    #[test]
    fn create_from_asset_creates_entity_with_sprite() {
        let mut engine = test_engine();
        let transform = Transform2D::from_position(Vec2::new(96.0, 48.0));

        let result = EditorCommand::create_from_asset_at("Tree", "sprites/tree.png", transform)
            .apply(&mut engine)
            .expect("command should succeed");

        let EditorCommandResult::CreatedNode(entity) = result else {
            panic!("create from asset should return the created entity id");
        };
        assert_eq!(
            engine.active_scene.node(entity).expect("node exists").name,
            "Tree"
        );
        assert_eq!(
            engine
                .active_scene
                .node(entity)
                .expect("node exists")
                .transform,
            transform
        );
        assert_eq!(
            engine
                .active_scene
                .sprite(entity)
                .expect("sprite exists")
                .sprite_path,
            "sprites/tree.png"
        );
    }

    #[test]
    fn create_camera_creates_entity_with_camera_component() {
        let mut engine = test_engine();

        let result = EditorCommand::create_camera("MainCamera")
            .apply(&mut engine)
            .expect("command should succeed");

        let EditorCommandResult::CreatedNode(entity) = result else {
            panic!("create camera should return the created entity id");
        };
        assert_eq!(
            engine.active_scene.node(entity).expect("node exists").name,
            "MainCamera"
        );
        assert!(engine.active_scene.camera(entity).is_some());
    }

    #[test]
    fn create_prefab_from_entity_then_instantiate_round_trip() {
        use crab2d_scene::Transform2D;

        let mut engine = test_engine();
        let original = engine.active_scene.spawn_node("Hero");
        engine
            .active_scene
            .add_sprite(
                original,
                crab2d_scene::SpriteComponent::new("sprites/hero.png"),
            )
            .expect("sprite should attach");

        EditorCommand::create_prefab_from_entity(original, "HeroPrefab")
            .apply(&mut engine)
            .expect("create prefab should succeed");

        assert!(engine.prefabs.get("HeroPrefab").is_some());

        let result = EditorCommand::instantiate_prefab("HeroPrefab", Transform2D::default())
            .apply(&mut engine)
            .expect("instantiate should succeed");

        let EditorCommandResult::CreatedNode(new_entity) = result else {
            panic!("instantiate should return the new entity id");
        };
        assert!(engine.active_scene.sprite(new_entity).is_some());
        assert_eq!(
            engine
                .active_scene
                .sprite(new_entity)
                .expect("sprite exists")
                .sprite_path,
            "sprites/hero.png"
        );
    }

    #[test]
    fn instantiate_unknown_prefab_returns_prefab_not_found() {
        use crab2d_scene::Transform2D;

        let mut engine = test_engine();

        let result = EditorCommand::instantiate_prefab("DoesNotExist", Transform2D::default())
            .apply(&mut engine);

        assert_eq!(result, Err(EditorCommandError::PrefabNotFound));
    }

    fn test_engine() -> Engine {
        Engine::new(EngineConfig::new("Crab2D Test"))
    }

    fn test_tilemap() -> TilemapComponent {
        TilemapComponent::new(TilemapSize::new(4, 4), TileSize::new(32, 32))
            .expect("tilemap should be valid")
    }
}
