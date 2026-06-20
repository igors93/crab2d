#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RenderStats {
    pub draw_calls: u32,
    pub sprites: u32,
    pub tilemaps: u32,
}

impl RenderStats {
    pub const ZERO: Self = Self {
        draw_calls: 0,
        sprites: 0,
        tilemaps: 0,
    };

    pub const fn new(draw_calls: u32, sprites: u32, tilemaps: u32) -> Self {
        Self {
            draw_calls,
            sprites,
            tilemaps,
        }
    }
}
