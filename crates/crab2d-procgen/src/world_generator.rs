use crate::GenerationSettings;
use crab2d_scene::Scene;

pub trait WorldGenerator {
    fn generate_scene(&self, settings: &GenerationSettings) -> Scene;
}
