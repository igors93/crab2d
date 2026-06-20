# Crab2D Architecture

Crab2D starts as a modular workspace. Each crate has one primary responsibility.

## Crates

| Crate | Responsibility |
|---|---|
| `crab2d-core` | Engine orchestration, project metadata, runtime coordination |
| `crab2d-scene` | Scene data, nodes, transforms, serializable components |
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

The workspace currently has two app entrypoints:

- `apps/crab2d-editor` edits and saves projects.
- `apps/crab2d-runtime` opens a saved project and runs it outside the editor.

The editor tracks project state through `EditorProjectSession`, which owns the
current project file path, project root, displayed project name, dirty flag, and
project asset roots. UI flows should ask the session/app where to save or load;
they should not hardcode `project.crab2d.json` except as the default file name.

## Runtime Boundary

Runtime gameplay data lives in `crab2d-scene` as serializable components.
Runtime scheduling lives in `crab2d-core`; `Engine::tick_with_input` runs scene
systems and returns a `FrameStep` with movement, collision, trigger, and camera
results. Platform input is represented by `crab2d-platform::InputState`, a small
data type that lets the core systems stay testable without window APIs.

Rendering is split the same way: `crab2d-render` can extract a `RenderList` from
scene data without owning the scene or knowing about the editor UI. Backends can
implement `Renderer2D` against that command list later.

Editor mutations should continue to pass through `EditorCommand` and
`CommandHistory`. This keeps no-code presets, future AI assistance, and eventual
Rust behavior tooling behind one auditable and undoable boundary.

## Current Core Modules

| File | Purpose |
|---|---|
| `crates/crab2d-core/src/config.rs` | Engine configuration |
| `crates/crab2d-core/src/engine.rs` | Engine state and runtime coordination |
| `crates/crab2d-core/src/project.rs` | Project identity and metadata |
| `crates/crab2d-core/src/lib.rs` | Public exports for the crate |
| `crates/crab2d-core/src/runtime_systems.rs` | Minimal scene systems for input, movement, AABB/tile collisions, triggers, and camera follow |
