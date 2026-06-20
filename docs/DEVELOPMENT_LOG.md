# Crab2D Development Log

This file records what we build and why. It is intentionally simple so the
project history stays readable without needing external tools.

## 2026-06-20

### Created the initial workspace

We created a Rust workspace with separate crates for:

- `crab2d-core`
- `crab2d-scene`
- `crab2d-render`
- `crab2d-editor`
- `crab2d-platform`
- `crab2d-assets`
- `crab2d-plugin-api`
- `crab2d-procgen`

Reason: keep the MVP small while giving the engine room to grow.

### Verified the first editor app

The command below successfully compiled and ran the editor app:

```bash
cargo run -p crab2d-editor-app
```

Observed output:

```text
Crab2D Editor opened 'Untitled Crab2D Project' in Select mode: 1 draw call(s), 3 visible node(s)
```

### Established the project philosophy

We documented the first version of the Crab2D philosophy in
`docs/PROJECT_PHILOSOPHY.md`.

Reason: future decisions should be measured against the same product direction.

### Polished the editor UI foundation

We added a small editor design system in the app layer:

- `apps/crab2d-editor/src/editor_theme.rs`
- `apps/crab2d-editor/src/editor_widgets.rs`

The editor UI now uses shared colors, spacing, panel headers, toolbar buttons,
tabs, chips, inspector sections, and asset cards. The main editor screen was
reworked around clearer toolbar groups, segmented scene/library navigation,
bottom dock tabs, a cleaner viewport overlay, structured inspector sections,
and a more usable image asset browser.

Reason: keep editor presentation modular and consistent without moving UI
concerns into runtime crates.

### Added a minimal playable runtime foundation

We added serializable runtime components to `crab2d-scene`:

- `Velocity2DComponent`
- `Collider2DComponent`
- `Aabb2D`

`Engine::tick` now runs a small scene-system pass that moves velocity-driven
nodes and reports AABB collision events through `FrameStep`. Invalid frame
deltas return an explicit `EngineTickError` instead of being ignored.

`crab2d-platform` now exposes `InputState` built from `PlatformEvent`, including
pressed, just-pressed, just-released keys, and cursor position. This keeps input
testable and outside the core engine.

`crab2d-render` now extracts a `RenderList` with camera, sprite, and tilemap
commands before stats are reported. This keeps the renderer backend boundary
small while making the draw pipeline more useful than a sprite counter.

Reason: make Crab2D minimally viable for simple 2D gameplay prototypes while
preserving clean editor/runtime boundaries.

### Closed the playable runtime MVP loop

We added a separate runtime app:

```bash
cargo run -p crab2d-runtime-app -- project.crab2d.json
```

The runtime opens a real window, loads a saved `ProjectDocument`, converts
keyboard input into `InputState`, ticks the engine, and draws the scene from
`RenderList` using an isolated `egui` renderer backend.

The scene/runtime model now includes:

- `PlayerControllerComponent`
- kinematic AABB collision resolution
- solid tilemap collision metadata through `TilesetCollision`
- `CameraFollowComponent`
- sensor trigger events through `TriggerComponent`

The editor inspector can edit the MVP gameplay components through
`EditorCommand` and `CommandHistory`, preserving the command boundary used by
save/load and undo/redo.

Reason: complete the smallest real game loop without coupling runtime behavior
to editor UI code. Audio, scripting, animation, and richer physics remain future
growth steps.

### Added real editor project workflow

The editor now tracks an `EditorProjectSession` with project path, project root,
project name, asset roots, and dirty/clean state. The app supports:

- New Project with templates and project folder creation
- Open Project from a `.crab2d.json` path
- Save and Save As with explicit status
- Play current saved project in `crab2d-runtime-app`
- gameplay presets for no-code creation
- add/remove component actions through `EditorCommand`

The starter tilemap no longer references a missing tileset asset by default; it
uses the editor/runtime fallback tile palette unless a real tileset is assigned.

Reason: make Crab2D usable as a project editor instead of a fixed demo scene,
while preserving command-based undo/redo and a clean path for future AI and Rust
behavior workflows.

### Added 10 engine systems

Added the following systems to bring Crab2D from a minimal prototype to a more
complete foundation for 2D games:

1. **Behavior scripting** — `BehaviorComponent` + `ScriptRuntime` backed by [Rhai](https://rhai.rs).
   Scripts live in `.rhai` files and expose `on_start`, `on_update(dt)`, `on_trigger(name)`.
   Output variables (`set_vel_x`, `destroy`, `load_scene`, …) communicate results back to
   the engine without unsafe Rust coupling.

2. **Audio** — `AudioComponent` + `AudioSystem` backed by rodio. Supports WAV and OGG,
   looping, volume, and auto-play on spawn. Gracefully no-ops when no audio device is
   available (e.g. headless CI).

3. **Sprite animation** — `AnimationComponent` with named states, per-state frame lists,
   and configurable FPS. `AnimationSystem::tick_animations` advances frames each tick.
   `current_uv()` returns the UV rect for the current frame in a spritesheet.

4. **In-game UI** — `UiLabelComponent` and `UiPanelComponent` for screen-space HUD
   elements. Both support a `UiAnchor` (9 positions) and pixel offsets. Rendered after
   the world pass.

5. **Scene manager** — `SceneManager` with a scene stack supporting `load_scene`,
   `push_scene`, `pop_scene`, and `restart`. Scenes are loaded from JSON by path.
   Transitions are applied at the start of the next frame.

6. **Asset pipeline** — `AssetRegistry` with UUID-based `AssetHandle`, directory scan,
   extension-based `AssetKind` detection, and JSON persistence for the registry itself.

7. **Save / Load** — `SaveData` key-value store (int, float, bool, string) and
   `GameSave` with numbered JSON save slots at `saves/save_NN.json`.

8. **Physics improvements** — `Collider2DComponent` gains `collision_layer` / `collision_mask`
   bitmasks, `one_way` platform flag, and `gravity_scale`. `PhysicsSettings` at scene level
   controls gravity direction, magnitude, and terminal velocity. Gravity is applied
   per-entity each tick based on `gravity_scale`.

9. **Particles** — `ParticleEmitterComponent` with emit rate, cone spread, speed range,
   per-particle gravity, color-over-lifetime, and size-over-lifetime. `ParticleSystem`
   manages runtime-only `ParticleState` (not serialized) keyed by entity ID.

10. **Procedural generation** — `StarterVillageGenerator` now produces a real `Scene`
    with a terrain grid, a wobbling river, randomized houses (wall + floor tiles), a
    dirt road, and a spawned player node at world-center. `GenerationSettings` controls
    map size, tile size, and seed.

Reason: these ten systems cover the most common needs for small-to-medium 2D games
while preserving the clean editor/runtime/scene boundary established in earlier work.
