# Runtime MVP

Crab2D now has a playable runtime MVP for small top-down prototypes. The loop is
kept intentionally small:

```text
InputState -> PlayerController -> Velocity -> collision resolution -> camera follow -> RenderList
```

The runtime app is separate from the editor and can open saved projects:

```bash
cargo run -p crab2d-runtime-app -- project.crab2d.json
```

## What Runs Per Tick

`Engine::tick(delta_seconds)` runs scene systems with empty input.
`Engine::tick_with_input(delta_seconds, input)` runs the playable path and
returns a `FrameStep`:

- reads `PlayerControllerComponent` and `InputState`
- writes movement into `Velocity2DComponent`
- moves nodes that have `Velocity2DComponent`
- detects AABB overlaps between `Collider2DComponent` instances
- resolves non-sensor AABB collisions by moving on X, resolving, then moving on Y
- filters entity collisions and triggers with `collision_layer` / `collision_mask`
- supports `one_way` entity platforms for downward Y-axis collision only
- resolves against solid tilemap cells from `TilesetCollision`
- reports sensor/trigger events through `TriggerEvent`
- applies `CameraFollowComponent`
- reports explicit `EngineTickError::InvalidDelta` for non-finite or negative
  frame deltas

The core systems accept `InputState` as data, so they remain testable without
opening a window.

## Input Boundary

`crab2d-platform` owns `InputState`, which is derived from `PlatformEvent`.
It tracks:

- currently pressed keys
- keys pressed this frame
- keys released this frame
- cursor position

This keeps OS/window input separate from `crab2d-core` and lets headless tests
drive gameplay without a real window.

## Scene Data

Runtime components live in `crab2d-scene` and serialize with `ProjectDocument`:

- `Velocity2DComponent`
- `Collider2DComponent`
- `PlayerControllerComponent`
- `CameraFollowComponent`
- `TriggerComponent`
- `TilesetCollision` inside `TilemapComponent`

The starter scene gives the player both components, so saved starter projects
already contain the minimum data needed for movement, camera follow, collision,
and a trigger example.

## Render Boundary

`crab2d-render` can extract a `RenderList` from a `Scene`. It contains:

- active camera command
- sprite draw commands
- tilemap draw commands

`apps/crab2d-runtime` uses an isolated `egui` painter backend for the MVP. It
opens a real window, loads sprite and tileset images from the project/assets
paths, applies camera transforms, respects render order from `RenderList`, and
falls back to colored tiles when image assets are missing.

`NullRenderer` still avoids opening a graphics backend and remains useful for
tests or headless validation.

## Intentional Limits

This is not a full physics engine. Collision is kinematic AABB only: no forces,
mass, rotations, polygons, or rigid bodies. Entity colliders support layer/mask
filtering and one-way platform behavior; solid tilemap cells are still treated
as world geometry. Trigger events are reported in `FrameStep` and can be
forwarded to runtime scripts.

Audio is also outside this MVP. The next minimal step is a small audio command
API in `crab2d-render` or a dedicated audio crate, with runtime-only playback by
asset path and global volume.
