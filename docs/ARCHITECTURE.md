# Crab2D Architecture

Crab2D is a modular Rust workspace. Each crate has one primary responsibility.

## Crates

| Crate | Responsibility |
|---|---|
| `crab2d-core` | Engine tick, runtime systems, scripting, audio, particles, scene manager, save/load, asset pipeline |
| `crab2d-scene` | Scene graph, nodes, transforms, all serializable components |
| `crab2d-render` | Renderer abstraction (`RenderList`, `Renderer2D` trait, null backend) |
| `crab2d-editor` | Editor state, inspector, commands, document workflow, undo/redo |
| `crab2d-platform` | Windowing, input (`InputState`), headless shell, OS integration |
| `crab2d-assets` | Typed asset registry, asset handles, import helpers |
| `crab2d-procgen` | Procedural world generators (`StarterVillageGenerator`, `GridMap`) |
| `crab2d-plugin-api` | Public extension boundary for future tools and plugins |

## App entrypoints

| Binary | Purpose |
|---|---|
| `apps/crab2d-editor` | Full editor — create, edit, save, and run projects |
| `apps/crab2d-runtime` | Standalone runtime — open a `.crab2d.json` and play it |

## Dependency direction

Dependencies flow inward. The outer layers know about the inner ones; inner crates
never import editor or platform code.

```text
apps/crab2d-editor-app
  -> crab2d-editor (app crate)
       -> crab2d-core
            -> crab2d-scene
            -> crab2d-assets
            -> crab2d-plugin-api
       -> crab2d-render
       -> crab2d-platform
       -> crab2d-procgen

apps/crab2d-runtime-app
  -> crab2d-core
  -> crab2d-render
  -> crab2d-platform
  -> crab2d-scene
```

## Runtime boundary

Runtime gameplay data lives in `crab2d-scene` as serializable components.
Runtime scheduling lives in `crab2d-core`.

`Engine::tick_with_input` runs all active scene systems and returns a `FrameStep`
containing movement, collision, trigger, and camera results.

`InputState` from `crab2d-platform` is a plain data type, keeping systems fully
testable without window APIs.

Rendering is extracted through `RenderList::from_scene` without the renderer
owning the scene or knowing about editor UI. Backends implement `Renderer2D`
against that command list.

Editor mutations pass through `EditorCommand` and `CommandHistory`, keeping
undo/redo, future AI assistance, and behavior scripting behind one auditable
boundary.

## `crab2d-core` modules

| Module | Purpose |
|---|---|
| `engine.rs` | `Engine` struct — owns the active scene, drives tick |
| `config.rs` | `EngineConfig` — startup options |
| `project.rs` | `ProjectInfo`, `ProjectMetadata` |
| `project_document.rs` | JSON load/save for full project files |
| `runtime_systems.rs` | AABB collision, player input, tilemap solid tiles, camera follow, triggers |
| `animation_system.rs` | Advance spritesheet frame timers each tick |
| `particle_system.rs` | Spawn and age particles for all `ParticleEmitterComponent` entities |
| `script_runtime.rs` | Rhai engine wrapper — load `.rhai` files, call `on_start`/`on_update`/`on_trigger` |
| `audio_system.rs` | rodio output stream wrapper — play WAV/OGG clips |
| `scene_manager.rs` | Scene stack — `load_scene`, `push_scene`, `pop_scene`, `restart` |
| `save_system.rs` | `SaveData` key-value store, `GameSave` slot-based JSON persistence |
| `asset_pipeline.rs` | `AssetRegistry`, `AssetHandle` (UUID-based), directory scan, `.meta` concept |

## `crab2d-scene` components

See `docs/COMPONENTS.md` for the full component reference with field tables.

New component types follow this checklist:

1. New file `crates/crab2d-scene/src/<name>_component.rs`
2. Entry in `SceneComponents` (field + CRUD methods)
3. Accessor methods in `Scene` (`add_*`, `remove_*`, `*`, `*_mut`, `*s`)
4. Export in `crates/crab2d-scene/src/lib.rs`
5. Inspector section in `apps/crab2d-editor/src/editor_ui.rs`
6. New variant in `EditorComponentKind` and match arm in `snapshot_has_component`

## Project file format

Projects are saved as `project.crab2d.json`:

```json
{
  "info": { "name": "My Game", ... },
  "assets": { ... },
  "scene": {
    "name": "Main",
    "nodes": [...],
    "components": {
      "sprites": { "0": { "sprite_path": "sprites/player.png" } },
      "colliders": { ... },
      ...
    },
    "physics_settings": { "gravity": [0, -980], "enabled": false }
  }
}
```

All component maps use `#[serde(default)]` so older saved files remain loadable
when new component types are added.
