use crate::{GenerationSettings, GridMap, TileKind, WorldGenerator};

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

#[cfg(test)]
mod tests {
    use crate::{GenerationSettings, StarterVillageGenerator, TileKind, WorldGenerator};

    #[test]
    fn starter_village_contains_road_and_water_guides() {
        let generator = StarterVillageGenerator;
        let map = generator.generate(GenerationSettings::new(1, 9, 9));

        assert_eq!(map.get(0, 4), Some(TileKind::Road));
        assert_eq!(map.get(3, 0), Some(TileKind::Water));
    }
}
