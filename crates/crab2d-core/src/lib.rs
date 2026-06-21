mod config;
mod engine;
pub mod game_flow;
pub mod playtest;
mod project;
mod project_document;
mod runtime_systems;

pub mod animation_system;
pub mod asset_pipeline;
pub mod audio_system;
pub mod particle_system;
pub mod save_system;
pub mod scene_manager;
pub mod script_runtime;

pub use config::EngineConfig;
pub use crab2d_scene::{PrefabRegistry, PrefabTemplate};
pub use engine::Engine;
pub use game_flow::GameFlow;
pub use playtest::{DiagnosticKind, PlaytestState, PlaytestStatus, RuntimeDiagnostic};
pub use project::{ProjectInfo, ProjectMetadata};
pub use project_document::{ProjectDocument, ProjectIoError};
pub use runtime_systems::{
    CollisionAxis, CollisionEvent, CollisionResolution, EngineTickError, FrameStep,
    SolidCollisionEvent, SolidObstacle, TriggerEvent,
};
