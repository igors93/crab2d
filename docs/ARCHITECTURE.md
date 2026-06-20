# Crab2D Architecture

Crab2D starts as a modular workspace. Each crate has one primary responsibility.

## Crates

| Crate | Responsibility |
|---|---|
| `crab2d-core` | Engine orchestration, project metadata, runtime coordination |
| `crab2d-scene` | Scene data, nodes, transforms, future components |
| `crab2d-render` | Renderer abstraction and future graphics backends |
| `crab2d-editor` | Editor application state and editor-only workflows |
| `crab2d-platform` | Windowing, input, filesystem dialogs, OS integration |
| `crab2d-assets` | Asset IDs, asset registry, future import pipeline |
| `crab2d-plugin-api` | Public extension boundary for future tools and plugins |
| `crab2d-procgen` | Future procedural generation systems |

## Dependency Direction

The project should keep dependencies flowing inward:

```text
apps/crab2d-editor
  -> crab2d-editor
    -> crab2d-core
      -> crab2d-scene
      -> crab2d-assets
      -> crab2d-plugin-api
```

Renderer and platform crates should stay behind abstractions so the editor can
change its UI/windowing implementation without rewriting scene or project data.

## Current Core Modules

| File | Purpose |
|---|---|
| `crates/crab2d-core/src/config.rs` | Engine configuration |
| `crates/crab2d-core/src/engine.rs` | Engine state and runtime coordination |
| `crates/crab2d-core/src/project.rs` | Project identity and metadata |
| `crates/crab2d-core/src/lib.rs` | Public exports for the crate |
