use std::error::Error;
use std::fmt;

use crate::TileKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridMap {
    pub width: u32,
    pub height: u32,
    tiles: Vec<TileKind>,
}

impl GridMap {
    pub fn new(width: u32, height: u32, fill: TileKind) -> Self {
        Self::try_new(width, height, fill).expect("grid map dimensions are too large")
    }

    pub fn try_new(width: u32, height: u32, fill: TileKind) -> Result<Self, GridMapError> {
        let tile_count = width
            .checked_mul(height)
            .ok_or(GridMapError::DimensionsTooLarge)?;

        Ok(Self {
            width,
            height,
            tiles: vec![fill; tile_count as usize],
        })
    }

    pub fn get(&self, x: u32, y: u32) -> Option<TileKind> {
        let index = self.index(x, y)?;
        self.tiles.get(index).copied()
    }

    pub fn set(&mut self, x: u32, y: u32, tile: TileKind) {
        if let Some(index) = self.index(x, y) {
            self.tiles[index] = tile;
        }
    }

    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }

    fn index(&self, x: u32, y: u32) -> Option<usize> {
        if x < self.width && y < self.height {
            Some((y * self.width + x) as usize)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridMapError {
    DimensionsTooLarge,
}

impl fmt::Display for GridMapError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DimensionsTooLarge => formatter.write_str("grid map dimensions are too large"),
        }
    }
}

impl Error for GridMapError {}

#[cfg(test)]
mod tests {
    use crate::{GridMap, GridMapError, TileKind};

    #[test]
    fn grid_map_stores_and_reads_tiles() {
        let mut map = GridMap::new(4, 3, TileKind::Grass);

        map.set(2, 1, TileKind::Water);

        assert_eq!(map.len(), 12);
        assert_eq!(map.get(2, 1), Some(TileKind::Water));
        assert_eq!(map.get(99, 99), None);
    }

    #[test]
    fn oversized_maps_are_rejected() {
        let result = GridMap::try_new(u32::MAX, u32::MAX, TileKind::Grass);

        assert_eq!(result, Err(GridMapError::DimensionsTooLarge));
    }
}
