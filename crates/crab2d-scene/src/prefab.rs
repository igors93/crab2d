use serde::{Deserialize, Serialize};

use crate::{
    AnimationComponent, AudioComponent, BehaviorComponent, Camera2DComponent, Collider2DComponent,
    EntityId, ParticleEmitterComponent, PlayerControllerComponent, Scene, SceneError,
    SpriteComponent, TagComponent, TilemapComponent, Transform2D, TriggerComponent,
    UiLabelComponent, UiPanelComponent, Velocity2DComponent, WorldTextComponent,
};

/// Captures the name and all components of a node so it can be stamped out many times.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrefabTemplate {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<TagComponent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sprite: Option<SpriteComponent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub velocity: Option<Velocity2DComponent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collider: Option<Collider2DComponent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_controller: Option<PlayerControllerComponent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub camera: Option<Camera2DComponent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub camera_follow_target_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger: Option<TriggerComponent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub behavior: Option<BehaviorComponent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<AudioComponent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animation: Option<AnimationComponent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui_label: Option<UiLabelComponent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui_panel: Option<UiPanelComponent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub particle_emitter: Option<ParticleEmitterComponent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_text: Option<WorldTextComponent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tilemap: Option<TilemapComponent>,
}

impl PrefabTemplate {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tag: None,
            sprite: None,
            velocity: None,
            collider: None,
            player_controller: None,
            camera: None,
            camera_follow_target_name: None,
            trigger: None,
            behavior: None,
            audio: None,
            animation: None,
            ui_label: None,
            ui_panel: None,
            particle_emitter: None,
            world_text: None,
            tilemap: None,
        }
    }

    /// Capture all components from an existing entity in the scene.
    pub fn from_entity(scene: &Scene, entity: EntityId) -> Option<Self> {
        let node = scene.node(entity)?;
        let mut prefab = Self::new(node.name.clone());
        prefab.tag = scene.tag(entity).cloned();
        prefab.sprite = scene.sprite(entity).cloned();
        prefab.velocity = scene.velocity(entity).cloned();
        prefab.collider = scene.collider(entity).cloned();
        prefab.player_controller = scene.player_controller(entity).cloned();
        prefab.camera = scene.camera(entity).cloned();
        prefab.camera_follow_target_name = scene
            .camera_follow(entity)
            .and_then(|cf| scene.node(cf.target).map(|n| n.name.clone()));
        prefab.trigger = scene.trigger(entity).cloned();
        prefab.behavior = scene.behavior(entity).cloned();
        prefab.audio = scene.audio(entity).cloned();
        prefab.animation = scene.animation(entity).cloned();
        prefab.ui_label = scene.ui_label(entity).cloned();
        prefab.ui_panel = scene.ui_panel(entity).cloned();
        prefab.particle_emitter = scene.particle_emitter(entity).cloned();
        prefab.world_text = scene.world_text(entity).cloned();
        prefab.tilemap = scene.tilemap(entity).cloned();
        Some(prefab)
    }

    /// Instantiate this prefab into the scene at the given transform.
    /// Returns the new entity's ID.
    pub fn instantiate(
        &self,
        scene: &mut Scene,
        transform: Transform2D,
    ) -> Result<EntityId, SceneError> {
        let entity = scene.spawn_node_with_transform(&self.name, transform)?;
        if let Some(c) = self.tag.clone() {
            let _ = scene.add_tag(entity, c);
        }
        if let Some(c) = self.sprite.clone() {
            let _ = scene.add_sprite(entity, c);
        }
        if let Some(c) = self.velocity {
            let _ = scene.add_velocity(entity, c);
        }
        if let Some(c) = self.collider {
            let _ = scene.add_collider(entity, c);
        }
        if let Some(c) = self.player_controller {
            let _ = scene.add_player_controller(entity, c);
        }
        if let Some(c) = self.camera {
            let _ = scene.add_camera(entity, c);
        }
        if let Some(c) = self.trigger.clone() {
            let _ = scene.add_trigger(entity, c);
        }
        if let Some(c) = self.behavior.clone() {
            let _ = scene.add_behavior(entity, c);
        }
        if let Some(c) = self.audio.clone() {
            let _ = scene.add_audio(entity, c);
        }
        if let Some(c) = self.animation.clone() {
            let _ = scene.add_animation(entity, c);
        }
        if let Some(c) = self.ui_label.clone() {
            let _ = scene.add_ui_label(entity, c);
        }
        if let Some(c) = self.ui_panel.clone() {
            let _ = scene.add_ui_panel(entity, c);
        }
        if let Some(c) = self.particle_emitter.clone() {
            let _ = scene.add_particle_emitter(entity, c);
        }
        if let Some(c) = self.world_text.clone() {
            let _ = scene.add_world_text(entity, c);
        }
        if let Some(c) = self.tilemap.clone() {
            let _ = scene.add_tilemap(entity, c);
        }
        Ok(entity)
    }
}

/// Flat list of named prefabs stored in the project document.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PrefabRegistry {
    pub templates: Vec<PrefabTemplate>,
}

impl PrefabRegistry {
    pub fn register(&mut self, prefab: PrefabTemplate) {
        if let Some(existing) = self.templates.iter_mut().find(|p| p.name == prefab.name) {
            *existing = prefab;
        } else {
            self.templates.push(prefab);
        }
    }

    pub fn get(&self, name: &str) -> Option<&PrefabTemplate> {
        self.templates.iter().find(|p| p.name == name)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        let before = self.templates.len();
        self.templates.retain(|p| p.name != name);
        self.templates.len() < before
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.templates.iter().map(|p| p.name.as_str())
    }
}
