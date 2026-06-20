# Crab2D Development Log

This file records what we build and why. It is intentionally simple so the
project history stays readable without needing external tools.

## 2026-06-20

### Created the initial workspace

We created a Rust workspace with separate crates for:

- `crab2d-core`
- `crab2d-scene`
- `crab2d-render`
- `crab2d-editor`
- `crab2d-platform`
- `crab2d-assets`
- `crab2d-plugin-api`
- `crab2d-procgen`

Reason: keep the MVP small while giving the engine room to grow.

### Verified the first editor app

The command below successfully compiled and ran the editor app:

```bash
cargo run -p crab2d-editor-app
```

Observed output:

```text
Crab2D Editor opened 'Untitled Crab2D Project' in Select mode: 1 draw call(s), 3 visible node(s)
```

### Established the project philosophy

We documented the first version of the Crab2D philosophy in
`docs/PROJECT_PHILOSOPHY.md`.

Reason: future decisions should be measured against the same product direction.

### Polished the editor UI foundation

We added a small editor design system in the app layer:

- `apps/crab2d-editor/src/editor_theme.rs`
- `apps/crab2d-editor/src/editor_widgets.rs`

The editor UI now uses shared colors, spacing, panel headers, toolbar buttons,
tabs, chips, inspector sections, and asset cards. The main editor screen was
reworked around clearer toolbar groups, segmented scene/library navigation,
bottom dock tabs, a cleaner viewport overlay, structured inspector sections,
and a more usable image asset browser.

Reason: keep editor presentation modular and consistent without moving UI
concerns into runtime crates.
