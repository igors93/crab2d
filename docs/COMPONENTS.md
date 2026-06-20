# Components

Components hold data only — no logic. Logic lives in systems (renderers, editors, future ECS schedulers).

## Core types

### `Node2D`

A named entity in the scene. Holds an `EntityId`, a `name: String`, and a `Transform2D`. Every spawned entity is a `Node2D`; components are attached to its `EntityId` separately.

### `Transform2D`

Position (`Vec2`), rotation in radians (`f32`), and uniform scale (`Vec2`). Defaults to the identity transform. Non-finite values are rejected at spawn time.

### `TagComponent`

A plain string label (`tag: String`) used to identify entities by role (e.g. `"player"`, `"world"`). Used by `Scene::find_node_by_tag`. Empty tags are rejected.

### `SpriteComponent`

Declares that an entity should be rendered as a sprite. Fields:

| Field         | Type     | Default |
|---------------|----------|---------|
| `sprite_path` | `String` | —       |
| `visible`     | `bool`   | `true`  |
| `z_index`     | `i32`    | `0`     |

Empty `sprite_path` is rejected by `Scene::add_sprite`. Serialized data that still
uses the old `asset_path` field is accepted as an alias during load, but new data
is written with `sprite_path`.

Future work: replace `sprite_path` with a typed `AssetId<Sprite>` after the asset
pipeline is ready to own scene references.

### `Camera2DComponent`

Marks an entity as the active camera. Fields:

| Field         | Type    | Default              |
|---------------|---------|----------------------|
| `zoom`        | `f32`   | `1.0`                |
| `clear_color` | `[f32; 4]` | `[0.08, 0.09, 0.10, 1.0]` |

Non-finite or non-positive `zoom` is rejected.

### `Velocity2DComponent`

Declares linear movement in world units per second. Fields:

| Field    | Type   | Default |
|----------|--------|---------|
| `linear` | `Vec2` | `Vec2::ZERO` |

The runtime tick applies `linear * delta_seconds` to the owning node's
`Transform2D::position`. Non-finite velocity values are rejected by
`Scene::add_velocity`.

`PlayerControllerComponent` can update this velocity every frame from keyboard
input.

### `Collider2DComponent`

Declares an axis-aligned rectangular collider. Fields:

| Field          | Type   | Default |
|----------------|--------|---------|
| `half_extents` | `Vec2` | —       |
| `offset`       | `Vec2` | `Vec2::ZERO` |
| `is_sensor`    | `bool` | `false` |

`Collider2DComponent::rectangle(size)` creates a collider from full width and
height. The runtime reports AABB overlaps in `FrameStep::collisions`. Non-sensor
colliders block movement through the kinematic resolver; sensor colliders only
report overlaps and trigger events.
Non-finite offsets or non-positive half extents are rejected by
`Scene::add_collider`.

### `PlayerControllerComponent`

Reads keyboard input into velocity for simple top-down movement. Fields:

| Field        | Type   | Default |
|--------------|--------|---------|
| `move_speed` | `f32`  | `160.0` |
| `enabled`    | `bool` | `true`  |

W/A/S/D and arrow keys map to movement. Diagonal input is normalized so diagonal
movement is not faster than straight movement. Negative, infinite, or NaN
`move_speed` values are rejected.

### `CameraFollowComponent`

Moves a camera node toward a target node. Fields:

| Field       | Type       | Default |
|-------------|------------|---------|
| `target`    | `EntityId` | —       |
| `smoothing` | `f32`      | `0.0`   |
| `enabled`   | `bool`     | `true`  |

`smoothing = 0.0` follows instantly. Missing targets are ignored safely during
the runtime tick.

### `TriggerComponent`

Names a sensor interaction. Fields:

| Field   | Type     | Default |
|---------|----------|---------|
| `name`  | `String` | —       |
| `once`  | `bool`   | `false` |

Attach it to an entity that also has a sensor `Collider2DComponent`. When another
collider overlaps it, the runtime records a `TriggerEvent` in `FrameStep`.

### `TilesetCollision`

Stored inside `TilemapComponent` as `collision`. It contains a serializable
`BTreeSet<u32>` of tile indices that should behave as solid AABBs at runtime.
The runtime generates tile collision boxes directly from visible solid tiles
instead of creating thousands of scene components.

## How `Scene` associates components to entities

`Scene` owns a `Vec<Node2D>` (the nodes) and a `SceneComponents` (the storage).
`SceneComponents` uses a `BTreeMap<EntityId, T>` per component type. Looking up
a component for a known entity id is `O(log n)`.

```
Scene
├── nodes: Vec<Node2D>             ← identity + transform
└── components: SceneComponents
    ├── tags:    BTreeMap<EntityId, TagComponent>
    ├── sprites: BTreeMap<EntityId, SpriteComponent>
    ├── cameras: BTreeMap<EntityId, Camera2DComponent>
    ├── tilemaps: BTreeMap<EntityId, TilemapComponent>
    ├── velocities: BTreeMap<EntityId, Velocity2DComponent>
    ├── colliders: BTreeMap<EntityId, Collider2DComponent>
    ├── player_controllers: BTreeMap<EntityId, PlayerControllerComponent>
    ├── camera_follows: BTreeMap<EntityId, CameraFollowComponent>
    └── triggers: BTreeMap<EntityId, TriggerComponent>
```

This flat, data-oriented layout keeps serialization straightforward (each map is independent) and paves the way for a future ECS scheduler without changing the storage contract.

## Growth path

- `sprite_path: String` → `sprite_path: TypedAssetId<Sprite>` once the asset pipeline owns scene references.
- Collision is currently kinematic AABB only; layers and richer responses can
  grow from `Collider2DComponent` and `FrameStep` without changing saved project
  structure.
- Behavior scripting should start as a small serializable `BehaviorComponent`
  that resolves to runtime-only Rust behavior code; see
  `docs/BEHAVIOR_SYSTEM_ROADMAP.md`.
- Additional component maps (audio, animation, gameplay state) follow the same
  `BTreeMap<EntityId, T>` pattern in `SceneComponents`.
- Editor and renderer only read from `Scene` — they never own it — keeping editor/runtime separation intact.
