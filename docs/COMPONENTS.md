# Components

Components hold data only — no logic. Logic lives in systems inside `crab2d-core`.

All components implement `Serialize` and `Deserialize` via serde. They are stored
in `SceneComponents` as `BTreeMap<EntityId, T>` — one map per component type.
Looking up a component for a known entity is `O(log n)`.

---

## Identity and transform

### `Node2D`

A named entity in the scene. Holds an `EntityId`, a `name: String`, and a
`Transform2D`. Every spawned entity is a `Node2D`; components are attached to its
`EntityId` separately.

### `Transform2D`

Position (`Vec2`), rotation in radians (`f32`), and scale (`Vec2`). Defaults to
the identity transform. Non-finite values are rejected at spawn time.

---

## Rendering

### `SpriteComponent`

| Field         | Type     | Default |
|---------------|----------|---------|
| `sprite_path` | `String` | —       |
| `visible`     | `bool`   | `true`  |
| `z_index`     | `i32`    | `0`     |

Empty `sprite_path` is rejected. Serialized data that still uses the legacy
`asset_path` field is accepted as an alias during load.

### `Camera2DComponent`

| Field         | Type       | Default                      |
|---------------|------------|------------------------------|
| `zoom`        | `f32`      | `1.0`                        |
| `clear_color` | `[f32; 4]` | `[0.08, 0.09, 0.10, 1.0]`   |

Non-finite or non-positive `zoom` is rejected.

### `AnimationComponent`

Drives spritesheet frame animation. Attaches named states, each with a frame list
and a target FPS.

| Field              | Type               | Default  |
|--------------------|--------------------|----|
| `spritesheet_path` | `String`           | —  |
| `frame_width`      | `u32`              | —  |
| `frame_height`     | `u32`              | —  |
| `columns`          | `u32`              | —  |
| `states`           | `Vec<AnimationState>` | `[]` |
| `current_state`    | `String`           | `""` |
| `current_frame`    | `u32`              | `0`  |
| `frame_timer`      | `f32`              | `0.0` |
| `playing`          | `bool`             | `true` |

`AnimationState` contains `name`, `frames: Vec<u32>` (tile indices), `fps`, and
`looping`. Call `set_state(name)` to switch states; the runtime advances frames
each tick.

---

## Physics

### `Velocity2DComponent`

| Field    | Type   | Default      |
|----------|--------|--------------|
| `linear` | `Vec2` | `Vec2::ZERO` |

Applied as `position += linear * delta_seconds`. Non-finite values are rejected.

### `Collider2DComponent`

Axis-aligned rectangular collider.

| Field             | Type   | Default    |
|-------------------|--------|------------|
| `half_extents`    | `Vec2` | —          |
| `offset`          | `Vec2` | `Vec2::ZERO` |
| `is_sensor`       | `bool` | `false`    |
| `collision_layer` | `u8`   | `1`        |
| `collision_mask`  | `u8`   | `0xFF`     |
| `one_way`         | `bool` | `false`    |
| `gravity_scale`   | `f32`  | `0.0`      |

`collision_layer` is a bitmask declaring which layer this entity occupies.
`collision_mask` declares which layers this entity collides against.
Entity-to-entity overlaps, triggers, and solid collision resolution require a
mutual match: each collider's mask must include the other collider's layer.
`one_way` makes a solid entity behave like a platform: it blocks only downward
Y-axis movement when the moving collider started above the platform top. Upward
movement from below and side entry are allowed.
`gravity_scale` multiplies the scene-level gravity. `0.0` disables gravity for
this entity.

Sensor colliders only report overlaps; non-sensor colliders block movement.

### `PhysicsSettings`

Scene-level physics configuration, stored directly on `Scene` (not per-entity).

| Field               | Type   | Default          |
|---------------------|--------|------------------|
| `gravity`           | `Vec2` | `(0, -980)` px/s² |
| `terminal_velocity` | `f32`  | `1200.0`         |
| `enabled`           | `bool` | `false`          |

`enabled = false` by default so top-down games are unaffected. Set to `true` for
platformers or any scene that needs downward pull.

### `TriggerComponent`

| Field  | Type     | Default |
|--------|----------|---------|
| `name` | `String` | —       |
| `once` | `bool`   | `false` |

Attach to a sensor collider. On overlap the runtime records a `TriggerEvent` in
`FrameStep`. Empty names are rejected.

---

## Input and movement

### `PlayerControllerComponent`

| Field        | Type   | Default |
|--------------|--------|---------|
| `move_speed` | `f32`  | `160.0` |
| `enabled`    | `bool` | `true`  |

W/A/S/D and arrow keys map to movement. Diagonal input is normalized.

### `CameraFollowComponent`

| Field      | Type       | Default |
|------------|------------|---------|
| `target`   | `EntityId` | —       |
| `smoothing`| `f32`      | `0.0`   |
| `enabled`  | `bool`     | `true`  |

`smoothing = 0.0` follows instantly. Missing targets are silently ignored.

---

## Tilemap

### `TilemapComponent`

Grid-based tile map with one or more named layers and a shared `TilesetCollision`
that marks which tile indices are solid at runtime.

`TilemapComponent::new(size, tile_size)` validates dimensions and creates the
default `"Ground"` layer. Solid tile metadata is stored in `collision` and is
consulted by the runtime AABB resolver.

---

## Scripting

### `BehaviorComponent`

Attaches a Rhai script to an entity.

| Field         | Type     | Default |
|---------------|----------|---------|
| `script_path` | `String` | —       |
| `enabled`     | `bool`   | `true`  |

The runtime loads the `.rhai` file from the project's `scripts/` directory and
calls `on_start()`, `on_update(dt)`, and `on_trigger(name)` as events occur.

Scripts communicate back via reserved output variables:

| Variable     | Type   | Effect                          |
|--------------|--------|---------------------------------|
| `set_vel_x`  | `f64`  | Override X velocity this frame  |
| `set_vel_y`  | `f64`  | Override Y velocity this frame  |
| `set_pos_x`  | `f64`  | Teleport X position             |
| `set_pos_y`  | `f64`  | Teleport Y position             |
| `destroy`    | `bool` | Despawn this entity             |
| `load_scene` | `String` | Trigger a scene transition    |

See the README for a full script example.

---

## Audio

### `AudioComponent`

| Field       | Type     | Default |
|-------------|----------|---------|
| `clip_path` | `String` | —       |
| `volume`    | `f32`    | `1.0`   |
| `looping`   | `bool`   | `false` |
| `auto_play` | `bool`   | `false` |
| `spatial`   | `bool`   | `false` |

Backed by rodio. WAV and OGG files are supported. `auto_play = true` starts
playback when the entity is spawned. Requires ALSA on Linux
(`libasound2-dev`).

---

## In-game UI

### `UiLabelComponent`

Screen-space text rendered after the world pass.

| Field         | Type       | Default          |
|---------------|------------|------------------|
| `text`        | `String`   | —                |
| `font_size`   | `f32`      | `16.0`           |
| `color_rgba`  | `[u8; 4]`  | `[255,255,255,255]` |
| `anchor`      | `UiAnchor` | `TopLeft`        |
| `offset_x/y`  | `f32`      | `0.0`            |
| `visible`     | `bool`     | `true`           |

### `UiPanelComponent`

Screen-space colored rectangle, useful for HUD backgrounds.

| Field        | Type       | Default            |
|--------------|------------|--------------------|
| `width/height` | `f32`    | —                  |
| `color_rgba` | `[u8; 4]`  | `[0,0,0,180]`      |
| `anchor`     | `UiAnchor` | `TopLeft`          |
| `offset_x/y` | `f32`      | `0.0`              |
| `visible`    | `bool`     | `true`             |

`UiAnchor` values: `TopLeft`, `TopCenter`, `TopRight`, `MiddleLeft`, `Center`,
`MiddleRight`, `BottomLeft`, `BottomCenter`, `BottomRight`.

---

## Particles

### `ParticleEmitterComponent`

| Field             | Type     | Default          |
|-------------------|----------|------------------|
| `texture_path`    | `String` | —                |
| `emit_rate`       | `f32`    | `20.0` /sec      |
| `particle_lifetime` | `f32`  | `1.0` sec        |
| `speed_min/max`   | `f32`    | `50.0` / `150.0` |
| `direction`       | `Vec2`   | `(0, 1)` up      |
| `spread_degrees`  | `f32`    | `30.0`           |
| `gravity_scale`   | `f32`    | `0.5`            |
| `size_start/end`  | `f32`    | `8.0` / `0.0`    |
| `color_start/end` | `[u8;4]` | yellow / red     |
| `enabled`         | `bool`   | `true`           |
| `max_particles`   | `u32`    | `256`            |

Particle runtime state (`ParticleState`, `Particle`) is not serialized — it is
rebuilt each time the scene is loaded.

---

## Storage layout

```
Scene
├── nodes: Vec<Node2D>             ← identity + transform
├── physics_settings: PhysicsSettings
└── components: SceneComponents
    ├── tags
    ├── sprites
    ├── cameras
    ├── tilemaps
    ├── velocities
    ├── colliders
    ├── player_controllers
    ├── camera_follows
    ├── triggers
    ├── behaviors
    ├── audios
    ├── animations
    ├── ui_labels
    ├── ui_panels
    └── particle_emitters
```

All maps are `BTreeMap<EntityId, T>`. Lookup is `O(log n)`. Serialization is
per-map and forward-compatible via `#[serde(default)]` on all new fields.

---

## Design rules

- Components never call methods on other components or scene APIs.
- All validation happens in `Scene::add_*`, not in the component constructor.
- Adding a new component type requires: new file in `crab2d-scene/src/`, entry
  in `SceneComponents`, accessors in `Scene`, export in `lib.rs`.
- Editor and renderer only read from `Scene` — they never own it.
