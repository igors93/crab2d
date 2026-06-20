# Crab2D

A modular 2D game engine and editor written in Rust. Designed to be comfortable for beginners and capable enough for larger projects.

## Features

**Editor**
- Scene hierarchy with node creation and selection
- Inspector with per-component sections (Transform, Sprite, Camera, Collider, Physics, Audio, Animation, UI, Particles, Script)
- Tilemap painter with solid/sensor tile support
- No-code gameplay presets (top-down player, wall, collectible, door)
- Save / Load / Save As / Run workflow with real project files
- Procedural world generation preview (starter village generator)

**Runtime**
- AABB collision with solid tilemap tiles
- Collision layers and masks (`collision_layer`, `collision_mask`)
- One-way platforms and per-entity gravity scale
- Player controller driven by WASD / arrow keys
- Camera follow with configurable smoothing
- Trigger / sensor events
- Sprite animation (spritesheet states, per-state FPS)
- Particle emitter with color lerp, size lerp, spread cone, gravity
- In-game UI labels and panels anchored to screen corners
- Scene manager with `load_scene`, `push_scene`, `pop_scene`
- Save / Load slots as JSON (`saves/save_00.json`)
- Behavior scripting via [Rhai](https://rhai.rs) (`.rhai` files, `on_start` / `on_update` / `on_trigger`)
- Audio playback via rodio (WAV / OGG, looping, auto-play)

**Asset pipeline**
- UUID-based `AssetHandle<T>` with `.meta` sidecar concept
- `AssetRegistry` with directory scan and path resolution
- Kind detection by file extension (image, audio, script, scene, font)

## Workspace Layout

```text
apps/
  crab2d-editor/          # editor executable (egui/eframe)
  crab2d-runtime/         # runtime executable for saved projects
crates/
  crab2d-core/            # engine tick, runtime systems, scripting, audio, particles
  crab2d-editor/          # editor state, commands, inspector, document workflow
  crab2d-platform/        # input, headless shell, OS integration
  crab2d-render/          # render list abstraction and null renderer
  crab2d-scene/           # scene graph, components, transforms
  crab2d-assets/          # typed asset registry and handles
  crab2d-procgen/         # procedural world generators
  crab2d-plugin-api/      # stable API boundary for plugins
```

## Quick Start

```bash
# Run the editor
cargo run -p crab2d-editor-app

# Run a saved project
cargo run -p crab2d-runtime-app -- project.crab2d.json

# Write the starter project to disk from the editor
cargo run -p crab2d-editor-app -- --save-starter-project
```

## Quality Checks

```bash
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

CI runs the same checks on every push to `main`.

> **Linux note:** rodio requires ALSA dev headers.
> Install with `sudo apt-get install libasound2-dev` before building.

## Project File

Projects are saved as `project.crab2d.json` and contain:

- `ProjectInfo` (name, version)
- `AssetRegistry` (registered asset paths and UUIDs)
- Active `Scene` (nodes, transforms, all components)

Scene files can also be loaded standalone by the runtime or the scene manager at runtime.

## Script API (Rhai)

Attach a `BehaviorComponent` pointing to a `.rhai` file. The engine calls these functions each frame:

```javascript
// Available globals: entity_id, pos_x, pos_y, vel_x, vel_y, tag, dt
// Write output globals to communicate back to the engine:
//   set_vel_x, set_vel_y, set_pos_x, set_pos_y, destroy, load_scene

fn on_start() {
    print(`entity ${entity_id} ready at (${pos_x}, ${pos_y})`);
}

fn on_update(dt) {
    if vel_y < -800.0 {
        set_vel_y = -800.0;
    }
}

fn on_trigger(name) {
    if name == "coin" {
        destroy = true;
    }
}
```

## Scene Manager

```rust
// From a script or runtime system:
scene_manager.load_scene("levels/level2.json");  // replace current
scene_manager.push_scene("ui/pause_menu.json");  // stack push
scene_manager.pop_scene();                        // return to previous
```

## Save System

```rust
let mut data = save.load(0)?;       // slot 0
data.set_int("coins", 42);
data.set_bool("boss_defeated", true);
save.save(0, &data)?;
```

## Procedural Generation

```rust
use crab2d_procgen::{GenerationSettings, StarterVillageGenerator, WorldGenerator};

let scene = StarterVillageGenerator.generate_scene(&GenerationSettings {
    scene_name: "World".into(),
    map_width: 64,
    map_height: 48,
    tile_size: 32,
    seed: Some(42),
});
```

## Component Reference

| Component | Crate | Purpose |
|---|---|---|
| `Transform2D` | scene | Position, rotation, scale |
| `SpriteComponent` | scene | Texture path, z-index, tint |
| `Camera2DComponent` | scene | Zoom, clear color |
| `CameraFollowComponent` | scene | Smooth camera tracking |
| `Velocity2DComponent` | scene | Linear velocity |
| `Collider2DComponent` | scene | AABB, layer/mask, one-way, gravity scale |
| `PlayerControllerComponent` | scene | Keyboard-driven movement |
| `TriggerComponent` | scene | Named sensor events |
| `TilemapComponent` | scene | Grid map with solid tiles |
| `AnimationComponent` | scene | Spritesheet states and FPS |
| `AudioComponent` | scene | Clip path, volume, loop, auto-play |
| `BehaviorComponent` | scene | Rhai script path |
| `UiLabelComponent` | scene | Screen-space text with anchor |
| `UiPanelComponent` | scene | Screen-space colored rectangle |
| `ParticleEmitterComponent` | scene | Spawn rate, color/size over lifetime |
| `PhysicsSettings` | scene | Scene-level gravity and terminal velocity |

## Documentation

| File | Contents |
|---|---|
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Crate responsibilities, dependency graph, module table for `crab2d-core`, project file format |
| [`docs/COMPONENTS.md`](docs/COMPONENTS.md) | Field tables for every component, storage layout, design rules for adding new types |
| [`docs/BEHAVIOR_SYSTEM_ROADMAP.md`](docs/BEHAVIOR_SYSTEM_ROADMAP.md) | No-code presets, Rhai script API reference, AI boundary design |
| [`docs/PROJECT_PHILOSOPHY.md`](docs/PROJECT_PHILOSOPHY.md) | Product principles — why decisions are made the way they are |
| [`docs/RUNTIME_MVP.md`](docs/RUNTIME_MVP.md) | Original minimal runtime loop design |
| [`docs/TILEMAP_AND_ASSET_BROWSER.md`](docs/TILEMAP_AND_ASSET_BROWSER.md) | Tilemap painter and asset browser implementation notes |
| [`docs/DEVELOPMENT_LOG.md`](docs/DEVELOPMENT_LOG.md) | Chronological record of what was built and why |
