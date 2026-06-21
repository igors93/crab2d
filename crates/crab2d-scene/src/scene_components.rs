use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    AnimationComponent, AudioComponent, BehaviorComponent, Camera2DComponent,
    CameraFollowComponent, Collider2DComponent, EntityId, ParticleEmitterComponent,
    PlayerControllerComponent, SpriteComponent, TagComponent, TilemapComponent, TriggerComponent,
    UiLabelComponent, UiPanelComponent, Velocity2DComponent, WorldTextComponent,
};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct SceneComponents {
    #[serde(default)]
    tags: BTreeMap<EntityId, TagComponent>,
    #[serde(default)]
    sprites: BTreeMap<EntityId, SpriteComponent>,
    #[serde(default)]
    cameras: BTreeMap<EntityId, Camera2DComponent>,
    #[serde(default)]
    tilemaps: BTreeMap<EntityId, TilemapComponent>,
    #[serde(default)]
    velocities: BTreeMap<EntityId, Velocity2DComponent>,
    #[serde(default)]
    colliders: BTreeMap<EntityId, Collider2DComponent>,
    #[serde(default)]
    player_controllers: BTreeMap<EntityId, PlayerControllerComponent>,
    #[serde(default)]
    camera_follows: BTreeMap<EntityId, CameraFollowComponent>,
    #[serde(default)]
    triggers: BTreeMap<EntityId, TriggerComponent>,
    #[serde(default)]
    behaviors: BTreeMap<EntityId, BehaviorComponent>,
    #[serde(default)]
    audios: BTreeMap<EntityId, AudioComponent>,
    #[serde(default)]
    animations: BTreeMap<EntityId, AnimationComponent>,
    #[serde(default)]
    ui_labels: BTreeMap<EntityId, UiLabelComponent>,
    #[serde(default)]
    ui_panels: BTreeMap<EntityId, UiPanelComponent>,
    #[serde(default)]
    particle_emitters: BTreeMap<EntityId, ParticleEmitterComponent>,
    #[serde(default)]
    world_texts: BTreeMap<EntityId, WorldTextComponent>,
}

impl SceneComponents {
    pub fn insert_tag(&mut self, entity: EntityId, component: TagComponent) {
        self.tags.insert(entity, component);
    }

    pub fn tag(&self, entity: EntityId) -> Option<&TagComponent> {
        self.tags.get(&entity)
    }

    pub fn remove_tag(&mut self, entity: EntityId) -> Option<TagComponent> {
        self.tags.remove(&entity)
    }

    pub fn insert_sprite(&mut self, entity: EntityId, component: SpriteComponent) {
        self.sprites.insert(entity, component);
    }

    pub fn sprite(&self, entity: EntityId) -> Option<&SpriteComponent> {
        self.sprites.get(&entity)
    }

    pub fn remove_sprite(&mut self, entity: EntityId) -> Option<SpriteComponent> {
        self.sprites.remove(&entity)
    }

    pub fn sprites(&self) -> impl Iterator<Item = (EntityId, &SpriteComponent)> {
        self.sprites
            .iter()
            .map(|(entity, component)| (*entity, component))
    }

    pub fn insert_camera(&mut self, entity: EntityId, component: Camera2DComponent) {
        self.cameras.insert(entity, component);
    }

    pub fn camera(&self, entity: EntityId) -> Option<&Camera2DComponent> {
        self.cameras.get(&entity)
    }

    pub fn remove_camera(&mut self, entity: EntityId) -> Option<Camera2DComponent> {
        self.cameras.remove(&entity)
    }

    pub fn insert_tilemap(&mut self, entity: EntityId, component: TilemapComponent) {
        self.tilemaps.insert(entity, component);
    }

    pub fn tilemap(&self, entity: EntityId) -> Option<&TilemapComponent> {
        self.tilemaps.get(&entity)
    }

    pub fn tilemap_mut(&mut self, entity: EntityId) -> Option<&mut TilemapComponent> {
        self.tilemaps.get_mut(&entity)
    }

    pub fn remove_tilemap(&mut self, entity: EntityId) -> Option<TilemapComponent> {
        self.tilemaps.remove(&entity)
    }

    pub fn tilemaps(&self) -> impl Iterator<Item = (EntityId, &TilemapComponent)> {
        self.tilemaps
            .iter()
            .map(|(entity, component)| (*entity, component))
    }

    pub fn insert_velocity(&mut self, entity: EntityId, component: Velocity2DComponent) {
        self.velocities.insert(entity, component);
    }

    pub fn velocity(&self, entity: EntityId) -> Option<&Velocity2DComponent> {
        self.velocities.get(&entity)
    }

    pub fn velocity_mut(&mut self, entity: EntityId) -> Option<&mut Velocity2DComponent> {
        self.velocities.get_mut(&entity)
    }

    pub fn remove_velocity(&mut self, entity: EntityId) -> Option<Velocity2DComponent> {
        self.velocities.remove(&entity)
    }

    pub fn velocities(&self) -> impl Iterator<Item = (EntityId, &Velocity2DComponent)> {
        self.velocities
            .iter()
            .map(|(entity, component)| (*entity, component))
    }

    pub fn insert_collider(&mut self, entity: EntityId, component: Collider2DComponent) {
        self.colliders.insert(entity, component);
    }

    pub fn collider(&self, entity: EntityId) -> Option<&Collider2DComponent> {
        self.colliders.get(&entity)
    }

    pub fn remove_collider(&mut self, entity: EntityId) -> Option<Collider2DComponent> {
        self.colliders.remove(&entity)
    }

    pub fn colliders(&self) -> impl Iterator<Item = (EntityId, &Collider2DComponent)> {
        self.colliders
            .iter()
            .map(|(entity, component)| (*entity, component))
    }

    pub fn insert_player_controller(
        &mut self,
        entity: EntityId,
        component: PlayerControllerComponent,
    ) {
        self.player_controllers.insert(entity, component);
    }

    pub fn player_controller(&self, entity: EntityId) -> Option<&PlayerControllerComponent> {
        self.player_controllers.get(&entity)
    }

    pub fn remove_player_controller(
        &mut self,
        entity: EntityId,
    ) -> Option<PlayerControllerComponent> {
        self.player_controllers.remove(&entity)
    }

    pub fn player_controllers(
        &self,
    ) -> impl Iterator<Item = (EntityId, &PlayerControllerComponent)> {
        self.player_controllers
            .iter()
            .map(|(entity, component)| (*entity, component))
    }

    pub fn insert_camera_follow(&mut self, entity: EntityId, component: CameraFollowComponent) {
        self.camera_follows.insert(entity, component);
    }

    pub fn camera_follow(&self, entity: EntityId) -> Option<&CameraFollowComponent> {
        self.camera_follows.get(&entity)
    }

    pub fn remove_camera_follow(&mut self, entity: EntityId) -> Option<CameraFollowComponent> {
        self.camera_follows.remove(&entity)
    }

    pub fn camera_follows(&self) -> impl Iterator<Item = (EntityId, &CameraFollowComponent)> {
        self.camera_follows
            .iter()
            .map(|(entity, component)| (*entity, component))
    }

    pub fn insert_trigger(&mut self, entity: EntityId, component: TriggerComponent) {
        self.triggers.insert(entity, component);
    }

    pub fn trigger(&self, entity: EntityId) -> Option<&TriggerComponent> {
        self.triggers.get(&entity)
    }

    pub fn remove_trigger(&mut self, entity: EntityId) -> Option<TriggerComponent> {
        self.triggers.remove(&entity)
    }

    pub fn triggers(&self) -> impl Iterator<Item = (EntityId, &TriggerComponent)> {
        self.triggers
            .iter()
            .map(|(entity, component)| (*entity, component))
    }

    // --- BehaviorComponent ---

    pub fn insert_behavior(&mut self, entity: EntityId, component: BehaviorComponent) {
        self.behaviors.insert(entity, component);
    }

    pub fn behavior(&self, entity: EntityId) -> Option<&BehaviorComponent> {
        self.behaviors.get(&entity)
    }

    pub fn behavior_mut(&mut self, entity: EntityId) -> Option<&mut BehaviorComponent> {
        self.behaviors.get_mut(&entity)
    }

    pub fn remove_behavior(&mut self, entity: EntityId) -> Option<BehaviorComponent> {
        self.behaviors.remove(&entity)
    }

    pub fn behaviors(&self) -> impl Iterator<Item = (EntityId, &BehaviorComponent)> {
        self.behaviors
            .iter()
            .map(|(entity, component)| (*entity, component))
    }

    // --- AudioComponent ---

    pub fn insert_audio(&mut self, entity: EntityId, component: AudioComponent) {
        self.audios.insert(entity, component);
    }

    pub fn audio(&self, entity: EntityId) -> Option<&AudioComponent> {
        self.audios.get(&entity)
    }

    pub fn audio_mut(&mut self, entity: EntityId) -> Option<&mut AudioComponent> {
        self.audios.get_mut(&entity)
    }

    pub fn remove_audio(&mut self, entity: EntityId) -> Option<AudioComponent> {
        self.audios.remove(&entity)
    }

    pub fn audios(&self) -> impl Iterator<Item = (EntityId, &AudioComponent)> {
        self.audios
            .iter()
            .map(|(entity, component)| (*entity, component))
    }

    // --- AnimationComponent ---

    pub fn insert_animation(&mut self, entity: EntityId, component: AnimationComponent) {
        self.animations.insert(entity, component);
    }

    pub fn animation(&self, entity: EntityId) -> Option<&AnimationComponent> {
        self.animations.get(&entity)
    }

    pub fn animation_mut(&mut self, entity: EntityId) -> Option<&mut AnimationComponent> {
        self.animations.get_mut(&entity)
    }

    pub fn remove_animation(&mut self, entity: EntityId) -> Option<AnimationComponent> {
        self.animations.remove(&entity)
    }

    pub fn animations(&self) -> impl Iterator<Item = (EntityId, &AnimationComponent)> {
        self.animations
            .iter()
            .map(|(entity, component)| (*entity, component))
    }

    // --- UiLabelComponent ---

    pub fn insert_ui_label(&mut self, entity: EntityId, component: UiLabelComponent) {
        self.ui_labels.insert(entity, component);
    }

    pub fn ui_label(&self, entity: EntityId) -> Option<&UiLabelComponent> {
        self.ui_labels.get(&entity)
    }

    pub fn ui_label_mut(&mut self, entity: EntityId) -> Option<&mut UiLabelComponent> {
        self.ui_labels.get_mut(&entity)
    }

    pub fn remove_ui_label(&mut self, entity: EntityId) -> Option<UiLabelComponent> {
        self.ui_labels.remove(&entity)
    }

    pub fn ui_labels(&self) -> impl Iterator<Item = (EntityId, &UiLabelComponent)> {
        self.ui_labels
            .iter()
            .map(|(entity, component)| (*entity, component))
    }

    // --- UiPanelComponent ---

    pub fn insert_ui_panel(&mut self, entity: EntityId, component: UiPanelComponent) {
        self.ui_panels.insert(entity, component);
    }

    pub fn ui_panel(&self, entity: EntityId) -> Option<&UiPanelComponent> {
        self.ui_panels.get(&entity)
    }

    pub fn ui_panel_mut(&mut self, entity: EntityId) -> Option<&mut UiPanelComponent> {
        self.ui_panels.get_mut(&entity)
    }

    pub fn remove_ui_panel(&mut self, entity: EntityId) -> Option<UiPanelComponent> {
        self.ui_panels.remove(&entity)
    }

    pub fn ui_panels(&self) -> impl Iterator<Item = (EntityId, &UiPanelComponent)> {
        self.ui_panels
            .iter()
            .map(|(entity, component)| (*entity, component))
    }

    // --- ParticleEmitterComponent ---

    pub fn insert_particle_emitter(
        &mut self,
        entity: EntityId,
        component: ParticleEmitterComponent,
    ) {
        self.particle_emitters.insert(entity, component);
    }

    pub fn particle_emitter(&self, entity: EntityId) -> Option<&ParticleEmitterComponent> {
        self.particle_emitters.get(&entity)
    }

    pub fn particle_emitter_mut(
        &mut self,
        entity: EntityId,
    ) -> Option<&mut ParticleEmitterComponent> {
        self.particle_emitters.get_mut(&entity)
    }

    pub fn remove_particle_emitter(
        &mut self,
        entity: EntityId,
    ) -> Option<ParticleEmitterComponent> {
        self.particle_emitters.remove(&entity)
    }

    pub fn particle_emitters(&self) -> impl Iterator<Item = (EntityId, &ParticleEmitterComponent)> {
        self.particle_emitters
            .iter()
            .map(|(entity, component)| (*entity, component))
    }

    // --- WorldTextComponent ---

    pub fn insert_world_text(&mut self, entity: EntityId, component: WorldTextComponent) {
        self.world_texts.insert(entity, component);
    }

    pub fn world_text(&self, entity: EntityId) -> Option<&WorldTextComponent> {
        self.world_texts.get(&entity)
    }

    pub fn world_text_mut(&mut self, entity: EntityId) -> Option<&mut WorldTextComponent> {
        self.world_texts.get_mut(&entity)
    }

    pub fn remove_world_text(&mut self, entity: EntityId) -> Option<WorldTextComponent> {
        self.world_texts.remove(&entity)
    }

    pub fn world_texts(&self) -> impl Iterator<Item = (EntityId, &WorldTextComponent)> {
        self.world_texts
            .iter()
            .map(|(entity, component)| (*entity, component))
    }

    pub fn remove_all(&mut self, entity: EntityId) {
        self.tags.remove(&entity);
        self.sprites.remove(&entity);
        self.cameras.remove(&entity);
        self.tilemaps.remove(&entity);
        self.velocities.remove(&entity);
        self.colliders.remove(&entity);
        self.player_controllers.remove(&entity);
        self.camera_follows.remove(&entity);
        self.triggers.remove(&entity);
        self.behaviors.remove(&entity);
        self.audios.remove(&entity);
        self.animations.remove(&entity);
        self.ui_labels.remove(&entity);
        self.ui_panels.remove(&entity);
        self.particle_emitters.remove(&entity);
        self.world_texts.remove(&entity);
    }
}
