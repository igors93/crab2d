# Crab2D Project Philosophy

Crab2D is a Rust-first 2D engine and editor built for small teams, solo
developers, and learners who want a clear path from prototype to shipped game.

## Core Principles

1. **2D is the product**
   Crab2D will not treat 2D as a secondary mode inside a 3D engine. Every major
   system should be designed around 2D games first.

2. **Small pieces, clear names**
   Code should live in modules with obvious names. A developer should understand
   where to place a feature before reading the whole codebase.

3. **Editor and runtime stay separate**
   The editor can be rich and helpful, but game runtime code must remain lean.
   A shipped game should not carry editor-only systems.

4. **Data should be easy to inspect**
   Scenes, projects, assets, and settings should eventually use formats that are
   friendly to version control and human review.

5. **Extensibility comes from stable boundaries**
   Plugins, custom tools, and future procedural systems should depend on small,
   intentional APIs rather than private engine internals.

6. **Correctness before cleverness**
   Crab2D should prefer predictable, testable systems over impressive code that
   is hard to maintain.

## What We Are Building First

The first version is not a full engine. It is a foundation:

- a workspace layout that can grow
- project metadata
- a minimal engine loop
- a scene model
- a placeholder renderer
- an editor entrypoint
- documentation that records decisions as the project evolves

## What We Are Avoiding For Now

- procedural generation
- plugin loading from dynamic libraries
- a real renderer backend
- physics
- scripting
- asset importing
- Steam export

Those features matter, but they should be added after the foundation has a clean
shape.
