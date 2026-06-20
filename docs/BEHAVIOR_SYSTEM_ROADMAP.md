# Behavior System Roadmap

Crab2D should support two creation modes without mixing editor UI, runtime data,
and gameplay execution.

## No-Code Mode

The current MVP uses visual scene components and editor presets:

- `Top Down Player`
- `Static Wall`
- `Collectible`
- `Door`
- `Trigger Area`
- `Camera Follow`

Presets are thin wrappers over `EditorCommand`. They create or update ordinary
serializable components such as `SpriteComponent`, `Collider2DComponent`,
`Velocity2DComponent`, `PlayerControllerComponent`, `CameraFollowComponent`, and
`TriggerComponent`.

This keeps saved projects easy to diff and lets undo/redo, future AI assistance,
and runtime validation share the same command boundary.

## Advanced Rust Mode

A future advanced mode can add a serializable component like:

```rust
pub struct BehaviorComponent {
    pub behavior_id: String,
    pub config_path: Option<String>,
}
```

The runtime would resolve `behavior_id` through a gameplay crate or behavior
registry, then call a stable API:

```rust
pub trait Crab2DBehavior {
    fn update(&mut self, ctx: &mut BehaviorContext);
}
```

The editor should not compile or mutate behavior code directly. It should create
project diffs and route advanced edits through explicit commands.

## Future AI Boundary

AI-assisted editing should operate like this:

```text
AI Assistant -> Permission Layer -> EditorCommand -> Project Diff -> User Approval
```

Candidate permissions:

- `ReadProject`
- `CreateProject`
- `EditScene`
- `CreateNode`
- `EditComponent`
- `ImportAsset`
- `WriteBehaviorCode`
- `RunProject`
- `BuildProject`
- `DeleteFile`

The current project/session work prepares for this by keeping project JSON
explicit, editor mutations command-based, and gameplay state serializable.

## Growth Steps

1. Add a `BehaviorComponent` data shape without executing code.
2. Add a behavior registry in runtime-only code.
3. Add a small `BehaviorContext` API for scene queries and commands.
4. Add generated Rust behavior crates per project.
5. Explore hot reload only after the stable API is useful without it.
