# Tilemap and Asset Browser

This change adds the first editor-ready world-building vertical slice.

## Runtime Data

`TilemapComponent` lives in `crab2d-scene` so saved projects can store tilemaps
without depending on editor UI code. The component is serializable and contains:

- map dimensions
- tile size
- optional tileset image metadata
- ordered layers
- sparse optional tile cells

The scene owns tilemaps through `SceneComponents`, the same boundary used by
sprites, tags, and cameras.

## Editor Commands

Tilemap editing goes through `EditorCommand`:

- `AttachTilemap`
- `SetTile`

`CommandHistory` stores the previous tile value, so painting and erasing are
undoable and redoable.

## Editor UI

The editor app scans image assets from the editor asset root and project
`assets/` directory. The bottom dock now shows real images when available, and
clicking an image applies it to the selected sprite node.

The viewport renders tilemaps before sprites. The Tile Brush and Erase tools
write into the active tilemap through editor commands, preserving undo/redo.

## Next Growth Step

The next natural step is a dedicated tileset importer that records tile size,
columns, rows, collision defaults, and preview thumbnails in `AssetRegistry`.
