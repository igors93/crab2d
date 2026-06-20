use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TilemapComponent {
    pub map_size: TilemapSize,
    pub tile_size: TileSize,
    pub tileset: Option<TilesetRef>,
    pub layers: Vec<TileLayer>,
}

impl TilemapComponent {
    pub fn new(map_size: TilemapSize, tile_size: TileSize) -> Result<Self, TilemapError> {
        if map_size.width == 0 || map_size.height == 0 {
            return Err(TilemapError::ZeroMapSize);
        }
        if tile_size.width == 0 || tile_size.height == 0 {
            return Err(TilemapError::ZeroTileSize);
        }

        let tile_count = map_size
            .width
            .checked_mul(map_size.height)
            .ok_or(TilemapError::MapTooLarge)?;

        Ok(Self {
            map_size,
            tile_size,
            tileset: None,
            layers: vec![TileLayer::new("Ground", tile_count as usize)?],
        })
    }

    pub fn with_tileset(mut self, tileset: TilesetRef) -> Result<Self, TilemapError> {
        tileset.validate()?;
        self.tileset = Some(tileset);
        Ok(self)
    }

    pub fn validate(&self) -> Result<(), TilemapError> {
        if self.map_size.width == 0 || self.map_size.height == 0 {
            return Err(TilemapError::ZeroMapSize);
        }
        if self.tile_size.width == 0 || self.tile_size.height == 0 {
            return Err(TilemapError::ZeroTileSize);
        }
        let tile_count = self
            .map_size
            .width
            .checked_mul(self.map_size.height)
            .ok_or(TilemapError::MapTooLarge)? as usize;
        if self.layers.is_empty() {
            return Err(TilemapError::MissingLayer);
        }
        if let Some(tileset) = &self.tileset {
            tileset.validate()?;
        }
        for layer in &self.layers {
            layer.validate(tile_count)?;
        }
        Ok(())
    }

    pub fn add_layer(&mut self, name: impl Into<String>) -> Result<(), TilemapError> {
        let tile_count = self.tile_count()?;
        self.layers.push(TileLayer::new(name, tile_count)?);
        Ok(())
    }

    pub fn layer(&self, name: &str) -> Option<&TileLayer> {
        self.layers.iter().find(|layer| layer.name == name)
    }

    pub fn layer_mut(&mut self, name: &str) -> Option<&mut TileLayer> {
        self.layers.iter_mut().find(|layer| layer.name == name)
    }

    pub fn tile(&self, layer_name: &str, x: u32, y: u32) -> Result<Option<TileCell>, TilemapError> {
        let index = self.index(x, y)?;
        let layer = self.layer(layer_name).ok_or(TilemapError::MissingLayer)?;
        Ok(layer.tiles[index])
    }

    pub fn set_tile(
        &mut self,
        layer_name: &str,
        x: u32,
        y: u32,
        tile: Option<TileCell>,
    ) -> Result<Option<TileCell>, TilemapError> {
        let index = self.index(x, y)?;
        let layer = self
            .layer_mut(layer_name)
            .ok_or(TilemapError::MissingLayer)?;
        let previous = layer.tiles[index];
        layer.tiles[index] = tile;
        Ok(previous)
    }

    pub fn tile_count(&self) -> Result<usize, TilemapError> {
        self.map_size
            .width
            .checked_mul(self.map_size.height)
            .map(|count| count as usize)
            .ok_or(TilemapError::MapTooLarge)
    }

    pub fn index(&self, x: u32, y: u32) -> Result<usize, TilemapError> {
        if x >= self.map_size.width || y >= self.map_size.height {
            return Err(TilemapError::OutOfBounds);
        }
        Ok((y * self.map_size.width + x) as usize)
    }

    pub fn visible_tiles(&self) -> Vec<VisibleTile<'_>> {
        let mut tiles = Vec::new();

        for layer in self.layers.iter().filter(|layer| layer.visible) {
            for (index, tile) in layer.tiles.iter().enumerate() {
                let Some(cell) = tile else {
                    continue;
                };
                let x = index as u32 % self.map_size.width;
                let y = index as u32 / self.map_size.width;
                tiles.push(VisibleTile { layer, x, y, cell });
            }
        }

        tiles.sort_by_key(|tile| tile.layer.z_index);
        tiles
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TilemapSize {
    pub width: u32,
    pub height: u32,
}

impl TilemapSize {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileSize {
    pub width: u32,
    pub height: u32,
}

impl TileSize {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileLayer {
    pub name: String,
    pub visible: bool,
    pub z_index: i32,
    pub tiles: Vec<Option<TileCell>>,
}

impl TileLayer {
    pub fn new(name: impl Into<String>, tile_count: usize) -> Result<Self, TilemapError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(TilemapError::EmptyLayerName);
        }
        Ok(Self {
            name,
            visible: true,
            z_index: 0,
            tiles: vec![None; tile_count],
        })
    }

    fn validate(&self, expected_tile_count: usize) -> Result<(), TilemapError> {
        if self.name.trim().is_empty() {
            return Err(TilemapError::EmptyLayerName);
        }
        if self.tiles.len() != expected_tile_count {
            return Err(TilemapError::LayerSizeMismatch);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileCell {
    pub tile_index: u32,
    pub tint_rgba: [u8; 4],
}

impl TileCell {
    pub const WHITE: [u8; 4] = [255, 255, 255, 255];

    pub const fn new(tile_index: u32) -> Self {
        Self {
            tile_index,
            tint_rgba: Self::WHITE,
        }
    }

    pub const fn with_tint(mut self, tint_rgba: [u8; 4]) -> Self {
        self.tint_rgba = tint_rgba;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TilesetRef {
    pub image_path: String,
    pub columns: u32,
    pub rows: u32,
}

impl TilesetRef {
    pub fn new(image_path: impl Into<String>, columns: u32, rows: u32) -> Self {
        Self {
            image_path: image_path.into(),
            columns,
            rows,
        }
    }

    fn validate(&self) -> Result<(), TilemapError> {
        if self.image_path.trim().is_empty() {
            return Err(TilemapError::EmptyTilesetPath);
        }
        if self.columns == 0 || self.rows == 0 {
            return Err(TilemapError::InvalidTilesetGrid);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisibleTile<'a> {
    pub layer: &'a TileLayer,
    pub x: u32,
    pub y: u32,
    pub cell: &'a TileCell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TilemapError {
    EmptyLayerName,
    EmptyTilesetPath,
    InvalidTilesetGrid,
    LayerSizeMismatch,
    MapTooLarge,
    MissingLayer,
    OutOfBounds,
    ZeroMapSize,
    ZeroTileSize,
}

impl fmt::Display for TilemapError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyLayerName => formatter.write_str("tile layer name cannot be empty"),
            Self::EmptyTilesetPath => formatter.write_str("tileset image path cannot be empty"),
            Self::InvalidTilesetGrid => {
                formatter.write_str("tileset grid must have at least one row and column")
            }
            Self::LayerSizeMismatch => {
                formatter.write_str("tile layer size does not match tilemap dimensions")
            }
            Self::MapTooLarge => formatter.write_str("tilemap dimensions are too large"),
            Self::MissingLayer => formatter.write_str("tile layer was not found"),
            Self::OutOfBounds => formatter.write_str("tile position is outside the tilemap bounds"),
            Self::ZeroMapSize => {
                formatter.write_str("tilemap dimensions must be greater than zero")
            }
            Self::ZeroTileSize => formatter.write_str("tile size must be greater than zero"),
        }
    }
}

impl Error for TilemapError {}

#[cfg(test)]
mod tests {
    use super::{TileCell, TileSize, TilemapComponent, TilemapError, TilemapSize, TilesetRef};

    #[test]
    fn tilemap_stores_and_replaces_tiles() {
        let mut tilemap = TilemapComponent::new(TilemapSize::new(4, 3), TileSize::new(16, 16))
            .expect("tilemap should be valid");

        let previous = tilemap
            .set_tile("Ground", 2, 1, Some(TileCell::new(7)))
            .expect("tile should write");
        let replaced = tilemap
            .set_tile("Ground", 2, 1, Some(TileCell::new(8)))
            .expect("tile should replace");

        assert_eq!(previous, None);
        assert_eq!(replaced, Some(TileCell::new(7)));
        assert_eq!(tilemap.tile("Ground", 2, 1), Ok(Some(TileCell::new(8))));
    }

    #[test]
    fn invalid_dimensions_are_rejected() {
        let result = TilemapComponent::new(TilemapSize::new(0, 3), TileSize::new(16, 16));

        assert_eq!(result, Err(TilemapError::ZeroMapSize));
    }

    #[test]
    fn tileset_metadata_is_validated() {
        let tilemap = TilemapComponent::new(TilemapSize::new(2, 2), TileSize::new(16, 16))
            .expect("tilemap should be valid");

        let result = tilemap.with_tileset(TilesetRef::new("", 0, 1));

        assert_eq!(result, Err(TilemapError::EmptyTilesetPath));
    }
}
