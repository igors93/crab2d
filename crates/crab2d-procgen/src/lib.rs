#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileKind {
    Grass,
    Water,
    Road,
    Forest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridMap {
    pub width: u32,
    pub height: u32,
    tiles: Vec<TileKind>,
}

impl GridMap {
    pub fn new(width: u32, height: u32, fill: TileKind) -> Self {
        Self {
            width,
            height,
            tiles: vec![fill; (width * height) as usize],
        }
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

    fn index(&self, x: u32, y: u32) -> Option<usize> {
        if x < self.width && y < self.height {
            Some((y * self.width + x) as usize)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GenerationSettings {
    pub seed: u64,
    pub width: u32,
    pub height: u32,
}

pub trait WorldGenerator {
    fn generate(&self, settings: GenerationSettings) -> GridMap;
}

#[derive(Debug, Default)]
pub struct StarterVillageGenerator;

impl WorldGenerator for StarterVillageGenerator {
    fn generate(&self, settings: GenerationSettings) -> GridMap {
        let mut map = GridMap::new(settings.width, settings.height, TileKind::Grass);

        for x in 0..settings.width {
            map.set(x, settings.height / 2, TileKind::Road);
        }

        for y in 0..settings.height {
            map.set(settings.width / 3, y, TileKind::Water);
        }

        map
    }
}
