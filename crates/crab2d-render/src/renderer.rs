use crab2d_scene::Scene;

use crate::RenderStats;

pub trait Renderer2D {
    fn begin_frame(&mut self);
    fn draw_scene(&mut self, scene: &Scene);
    fn end_frame(&mut self) -> RenderStats;
}
