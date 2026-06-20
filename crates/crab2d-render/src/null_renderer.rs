use crab2d_scene::Scene;

use crate::{RenderList, RenderStats, Renderer2D};

#[derive(Debug, Default)]
pub struct NullRenderer {
    render_list: RenderList,
}

impl NullRenderer {
    pub fn sprites_seen(&self) -> u32 {
        self.render_list
            .sprite_count()
            .try_into()
            .unwrap_or(u32::MAX)
    }

    pub fn tilemaps_seen(&self) -> u32 {
        self.render_list
            .tilemap_count()
            .try_into()
            .unwrap_or(u32::MAX)
    }

    pub fn render_list(&self) -> &RenderList {
        &self.render_list
    }
}

impl Renderer2D for NullRenderer {
    fn begin_frame(&mut self) {
        self.render_list = RenderList::default();
    }

    fn draw_scene(&mut self, scene: &Scene) {
        self.render_list = RenderList::from_scene(scene);
    }

    fn end_frame(&mut self) -> RenderStats {
        let sprites = self.sprites_seen();
        let tilemaps = self.tilemaps_seen();
        let draw_calls = self.render_list.items.len().try_into().unwrap_or(u32::MAX);
        RenderStats::new(draw_calls, sprites, tilemaps)
    }
}

#[cfg(test)]
mod tests {
    use crab2d_scene::{Scene, SpriteComponent, TileSize, TilemapComponent, TilemapSize};

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
        assert_eq!(stats.tilemaps, 0);
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
        assert_eq!(stats.tilemaps, 0);
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
        assert_eq!(stats.tilemaps, 0);
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
        assert_eq!(stats.tilemaps, 0);
    }

    #[test]
    fn tilemaps_are_counted() {
        let mut renderer = NullRenderer::default();
        let mut scene = Scene::new("Tilemap");
        let world = scene.spawn_node("World");
        scene
            .add_tilemap(
                world,
                TilemapComponent::new(TilemapSize::new(2, 2), TileSize::new(16, 16))
                    .expect("tilemap should be valid"),
            )
            .expect("tilemap should attach");

        renderer.begin_frame();
        renderer.draw_scene(&scene);
        let stats = renderer.end_frame();

        assert_eq!(stats.draw_calls, 1);
        assert_eq!(stats.sprites, 0);
        assert_eq!(stats.tilemaps, 1);
    }
}
