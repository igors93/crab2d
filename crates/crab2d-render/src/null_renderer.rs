use crab2d_scene::Scene;

use crate::{RenderStats, Renderer2D};

#[derive(Debug, Default)]
pub struct NullRenderer {
    sprites_seen: u32,
}

impl NullRenderer {
    pub fn sprites_seen(&self) -> u32 {
        self.sprites_seen
    }

    fn count_visible_nodes(scene: &Scene) -> u32 {
        scene.nodes().len().try_into().unwrap_or(u32::MAX)
    }
}

impl Renderer2D for NullRenderer {
    fn begin_frame(&mut self) {
        self.sprites_seen = 0;
    }

    fn draw_scene(&mut self, scene: &Scene) {
        self.sprites_seen = Self::count_visible_nodes(scene);
    }

    fn end_frame(&mut self) -> RenderStats {
        RenderStats::new(u32::from(self.sprites_seen > 0), self.sprites_seen)
    }
}

#[cfg(test)]
mod tests {
    use crab2d_scene::Scene;

    use crate::{NullRenderer, Renderer2D};

    #[test]
    fn empty_scene_produces_zero_stats() {
        let mut renderer = NullRenderer::default();
        let scene = Scene::new("Empty");

        renderer.begin_frame();
        renderer.draw_scene(&scene);
        let stats = renderer.end_frame();

        assert_eq!(stats.draw_calls, 0);
        assert_eq!(stats.sprites, 0);
    }

    #[test]
    fn scene_nodes_are_counted_as_visible_sprites() {
        let mut renderer = NullRenderer::default();
        let mut scene = Scene::new("Visible");
        scene.spawn_node("Player");
        scene.spawn_node("Camera2D");

        renderer.begin_frame();
        renderer.draw_scene(&scene);
        let stats = renderer.end_frame();

        assert_eq!(stats.draw_calls, 1);
        assert_eq!(stats.sprites, 2);
    }
}
