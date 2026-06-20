use crate::{GenerationSettings, GridMap};

pub trait WorldGenerator {
    fn generate(&self, settings: GenerationSettings) -> GridMap;
}
