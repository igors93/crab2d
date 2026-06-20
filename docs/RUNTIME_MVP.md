# Runtime MVP

Crab2D now has a small runtime foundation that is enough to build simple 2D
game prototypes without coupling gameplay data to the editor.

## What Runs Per Tick

`Engine::tick(delta_seconds)` runs scene systems and returns a `FrameStep`:

- moves nodes that have `Velocity2DComponent`
- detects AABB overlaps between `Collider2DComponent` instances
- reports explicit `EngineTickError::InvalidDelta` for non-finite or negative
  frame deltas

The tick does not read platform input directly. Apps should translate input into
scene data or gameplay commands first, then call `Engine::tick`.

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

The starter scene gives the player both components, so saved starter projects
already contain the minimum data needed for movement and collision checks.

## Render Boundary

`crab2d-render` can extract a `RenderList` from a `Scene`. It contains:

- active camera command
- sprite draw commands
- tilemap draw commands

`NullRenderer` still avoids opening a graphics backend, but it now stores the
render list and reports sprite/tilemap counts. This gives future GPU or software
renderers a small, stable draw-command API to implement.

## Intentional Limits

This is not a full physics engine yet. The current collision system reports
overlaps but does not resolve them. That keeps the first runtime API small and
testable while leaving a clear path for collision layers, trigger callbacks,
tilemap collision, and kinematic resolution.
