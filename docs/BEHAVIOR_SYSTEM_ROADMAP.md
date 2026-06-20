# Behavior System

Crab2D supports two creation modes that can coexist in the same project.

## No-Code Mode (implemented)

Visual scene components and editor presets create gameplay without writing code:

| Preset | Components attached |
|---|---|
| `Top Down Player` | `Velocity2D`, `Collider2D`, `PlayerController`, `Tag("player")` |
| `Static Wall` | `Collider2D` (solid) |
| `Collectible` | `Collider2D` (sensor), `TriggerComponent` |
| `Door` | `Collider2D` (sensor), `TriggerComponent` |
| `Trigger Area` | `Collider2D` (sensor), `TriggerComponent` |
| `Camera Follow` | `Camera2D`, `CameraFollow` |

Presets are thin wrappers over `EditorCommand`. They create ordinary serializable
components, so undo/redo, future AI assistance, and runtime validation all share
the same command boundary.

## Script Mode — Rhai (implemented)

Attach a `BehaviorComponent` with a path to a `.rhai` script file. The runtime
loads and runs the script using an embedded [Rhai](https://rhai.rs) interpreter.

### Script lifecycle

| Function | Called when |
|---|---|
| `on_start()` | Entity is first activated (scene load) |
| `on_update(dt)` | Every engine tick |
| `on_trigger(name)` | A `TriggerEvent` names this entity as an activator |

### Available globals (read)

| Variable    | Type  | Description |
|-------------|-------|-------------|
| `entity_id` | `i64` | This entity's numeric ID |
| `pos_x`     | `f64` | Current world X position |
| `pos_y`     | `f64` | Current world Y position |
| `vel_x`     | `f64` | Current X velocity |
| `vel_y`     | `f64` | Current Y velocity |
| `tag`       | `String` | Entity tag string (if any) |
| `dt`        | `f64` | Delta time in seconds (in `on_update`) |

### Output variables (write to communicate back)

| Variable     | Type     | Effect |
|--------------|----------|--------|
| `set_vel_x`  | `f64`    | Override X velocity this frame |
| `set_vel_y`  | `f64`    | Override Y velocity this frame |
| `set_pos_x`  | `f64`    | Teleport to X |
| `set_pos_y`  | `f64`    | Teleport to Y |
| `destroy`    | `bool`   | Despawn this entity at end of frame |
| `load_scene` | `String` | Replace current scene with this path |

### Example

```javascript
// scripts/bouncer.rhai

fn on_start() {
    print(`Bouncer ready at (${pos_x}, ${pos_y})`);
}

fn on_update(dt) {
    // Cap fall speed
    if vel_y < -600.0 {
        set_vel_y = -600.0;
    }
}

fn on_trigger(name) {
    if name == "goal" {
        load_scene = "levels/win_screen.json";
    }
    if name == "spike" {
        load_scene = "levels/level1.json";
    }
}
```

### Safety limits

- `Engine::set_max_operations(500_000)` prevents infinite loops from hanging the frame.
- Script errors are logged and silently skipped; they do not crash the runtime.
- Scripts cannot call native Rust functions unless explicitly registered on the `Engine`.

## AI Boundary (future)

AI-assisted editing should route through the same command boundary:

```text
AI Assistant → Permission Layer → EditorCommand → Project Diff → User Approval
```

Candidate permissions: `ReadProject`, `EditScene`, `CreateNode`, `EditComponent`,
`WriteBehaviorCode`, `RunProject`, `BuildProject`.

The current command-based mutation model and serializable project JSON already
support this pattern — no structural changes are needed to add the permission
layer.
