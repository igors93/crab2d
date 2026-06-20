# Crab2D

Crab2D is a modular Rust-first 2D game engine and editor prototype.

## Current Focus

The current focus is foundation work: simple code, clear module boundaries, and
project documentation that records why decisions are made.

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

## Documentation

- `docs/PROJECT_PHILOSOPHY.md` explains the product principles.
- `docs/ARCHITECTURE.md` explains the workspace boundaries.
- `docs/DEVELOPMENT_LOG.md` records what has been built and why.

## Quality Checks

Before committing, run the same checks used by CI:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

To run the editor app locally:

```bash
cargo run -p crab2d-editor-app
```

To write the starter project to `project.crab2d.json`:

```bash
cargo run -p crab2d-editor-app -- --save-starter-project
```

## Editor Viewport Assets

The editor app keeps development UI assets under `apps/crab2d-editor/assets`.
The starter scene uses `SpriteComponent::new("sprites/player.png")`, which the
editor first resolves relative to that asset root. If a file is not found there,
the editor tries the path relative to the current project directory.

This keeps saved scene data small and portable while allowing the editor to show
real textures during early development.

## Project Persistence

Project data can be saved as JSON using `ProjectDocument` from `crab2d-core`.
The default file name is `project.crab2d.json`, and the document currently stores:

- `ProjectInfo`
- `AssetRegistry`
- the active `Scene`
- `Node2D`, `Transform2D`, and scene components
