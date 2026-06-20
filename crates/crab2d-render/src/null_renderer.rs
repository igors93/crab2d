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

    fn count_visible_sprites(scene: &Scene) -> u32 {
        scene
            .sprites()
            .filter(|(_, sprite)| sprite.visible)
            .count()
            .try_into()
            .unwrap_or(u32::MAX)
    }
}

impl Renderer2D for NullRenderer {
    fn begin_frame(&mut self) {
        self.sprites_seen = 0;
    }

    fn draw_scene(&mut self, scene: &Scene) {
        self.sprites_seen = Self::count_visible_sprites(scene);
    }

    fn end_frame(&mut self) -> RenderStats {
        RenderStats::new(u32::from(self.sprites_seen > 0), self.sprites_seen)
    }
}

#[cfg(test)]
mod tests {
    use crab2d_scene::{Scene, SpriteComponent};

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
    fn scene_without_sprites_produces_zero_stats() {
        let mut renderer = NullRenderer::default();
        let mut scene = Scene::new("No Sprites");
        scene.spawn_node("Player");
        scene.spawn_node("Camera2D");

        renderer.begin_frame();
        renderer.draw_scene(&scene);
        let stats = renderer.end_frame();

        assert_eq!(stats.draw_calls, 0);
        assert_eq!(stats.sprites, 0);
    }

    #[test]
    fn visible_sprites_are_counted() {
        let mut renderer = NullRenderer::default();
        let mut scene = Scene::new("Visible");
        let player = scene.spawn_node("Player");
        scene
            .add_sprite(player, SpriteComponent::new("sprites/player.png"))
            .expect("sprite should attach");

        renderer.begin_frame();
        renderer.draw_scene(&scene);
        let stats = renderer.end_frame();

        assert_eq!(stats.draw_calls, 1);
        assert_eq!(stats.sprites, 1);
    }

    #[test]
    fn hidden_sprites_are_not_counted() {
        let mut renderer = NullRenderer::default();
        let mut scene = Scene::new("Hidden");
        let player = scene.spawn_node("Player");
        scene
            .add_sprite(player, SpriteComponent::new("sprites/player.png").hidden())
            .expect("sprite should attach");

        renderer.begin_frame();
        renderer.draw_scene(&scene);
        let stats = renderer.end_frame();

        assert_eq!(stats.draw_calls, 0);
        assert_eq!(stats.sprites, 0);
    }
}
