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

Empty `sprite_path` is rejected by `Scene::add_sprite`. Future work: replace `sprite_path` with a typed `AssetId<Sprite>`.

### `Camera2DComponent`

Marks an entity as the active camera. Fields:

| Field         | Type    | Default              |
|---------------|---------|----------------------|
| `zoom`        | `f32`   | `1.0`                |
| `clear_color` | `[f32; 4]` | `[0.1, 0.1, 0.1, 1.0]` |

Non-finite or non-positive `zoom` is rejected.

## How `Scene` associates components to entities

`Scene` owns a `Vec<Node2D>` (the nodes) and a `SceneComponents` (the storage). `SceneComponents` uses a `BTreeMap<EntityId, T>` per component type — tags, sprites, cameras. Looking up a component for a node is `O(log n)`.

```
Scene
├── nodes: Vec<Node2D>             ← identity + transform
└── components: SceneComponents
    ├── tags:    BTreeMap<EntityId, TagComponent>
    ├── sprites: BTreeMap<EntityId, SpriteComponent>
    └── cameras: BTreeMap<EntityId, Camera2DComponent>
```

This flat, data-oriented layout keeps serialization straightforward (each map is independent) and paves the way for a future ECS scheduler without changing the storage contract.

## Growth path

- `sprite_path: String` → `sprite_path: TypedAssetId<Sprite>` once the asset pipeline matures.
- Additional component maps (physics, audio, collider) follow the same `BTreeMap<EntityId, T>` pattern in `SceneComponents`.
- Editor and renderer only read from `Scene` — they never own it — keeping editor/runtime separation intact.
