#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GenerationSettings {
    pub seed: u64,
    pub width: u32,
    pub height: u32,
}

impl GenerationSettings {
    pub const fn new(seed: u64, width: u32, height: u32) -> Self {
        Self {
            seed,
            width,
            height,
        }
    }
}
