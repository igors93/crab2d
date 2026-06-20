# Crab2D

Crab2D is a modular Rust-first 2D game engine and editor prototype.

The first goal is intentionally small:

- load and save a 2D scene model
- render a basic 2D world
- run the scene from the editor
- export a desktop build later without changing the project layout

## Workspace Layout

```text
apps/
  crab2d-editor/          # executable entrypoint for the editor
crates/
  crab2d-core/            # engine orchestration and shared runtime types
  crab2d-editor/          # editor state, panels, commands, document workflow
  crab2d-platform/        # window, input, file dialogs, OS integration
  crab2d-render/          # renderer abstraction and future wgpu backend
  crab2d-scene/           # scene graph, entities, transforms, components
  crab2d-assets/          # asset registry, handles, import pipeline
  crab2d-procgen/         # procedural world generation APIs and generators
  crab2d-plugin-api/      # stable API boundary for internal/community plugins
```

## Suggested Growth Path

1. Keep gameplay data in `crab2d-scene`.
2. Keep editor-only behavior in `crab2d-editor`.
3. Keep graphics backend details in `crab2d-render`.
4. Keep operating system details in `crab2d-platform`.
5. Put user-facing extension points in `crab2d-plugin-api`.
6. Put world generation algorithms in `crab2d-procgen`.

This keeps the MVP small while allowing the editor, runtime, procedural tools,
and plugins to grow independently.
