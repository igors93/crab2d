use crab2d_scene::Scene;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderStats {
    pub draw_calls: u32,
    pub sprites: u32,
}

pub trait Renderer2D {
    fn begin_frame(&mut self);
    fn draw_scene(&mut self, scene: &Scene);
    fn end_frame(&mut self) -> RenderStats;
}

#[derive(Debug, Default)]
pub struct NullRenderer {
    sprites_seen: u32,
}

impl Renderer2D for NullRenderer {
    fn begin_frame(&mut self) {
        self.sprites_seen = 0;
    }

    fn draw_scene(&mut self, scene: &Scene) {
        self.sprites_seen = scene.nodes().len() as u32;
    }

    fn end_frame(&mut self) -> RenderStats {
        RenderStats {
            draw_calls: u32::from(self.sprites_seen > 0),
            sprites: self.sprites_seen,
        }
    }
}
