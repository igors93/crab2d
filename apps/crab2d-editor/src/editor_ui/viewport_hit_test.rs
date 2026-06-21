use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ViewportHitTarget {
    pub(super) entity: EntityId,
    pub(super) rect: egui::Rect,
}

pub(super) fn build_viewport_hit_targets(
    items: &[NodeView],
    world_to_screen: &dyn Fn(Vec2) -> egui::Pos2,
    viewport_zoom: f32,
    mut texture_size: impl FnMut(&str) -> Option<egui::Vec2>,
) -> Vec<ViewportHitTarget> {
    let mut targets = Vec::new();

    for item in items {
        if let Some(tilemap) = &item.tilemap {
            targets.push(ViewportHitTarget {
                entity: item.id,
                rect: tilemap_hit_rect(item, tilemap, world_to_screen, viewport_zoom),
            });
        }
    }

    for item in items {
        targets.push(ViewportHitTarget {
            entity: item.id,
            rect: node_hit_rect(item, world_to_screen, viewport_zoom, &mut texture_size),
        });
    }

    targets
}

pub(super) fn hit_target_at(
    targets: &[ViewportHitTarget],
    position: egui::Pos2,
    expansion: f32,
) -> Option<ViewportHitTarget> {
    targets
        .iter()
        .rev()
        .find(|target| target.rect.expand(expansion).contains(position))
        .copied()
}

pub(super) fn selected_hit_rect(
    targets: &[ViewportHitTarget],
    selected: Option<EntityId>,
) -> Option<egui::Rect> {
    let selected = selected?;
    targets
        .iter()
        .rev()
        .find(|target| target.entity == selected)
        .map(|target| target.rect)
}

pub(super) fn resize_handle_at(rect: egui::Rect, pos: egui::Pos2) -> Option<ResizeHandle> {
    resize_handle_rects(rect)
        .into_iter()
        .find(|(_, handle_rect)| handle_rect.contains(pos))
        .map(|(handle, _)| handle)
}

fn tilemap_hit_rect(
    item: &NodeView,
    tilemap: &TilemapComponent,
    world_to_screen: &dyn Fn(Vec2) -> egui::Pos2,
    viewport_zoom: f32,
) -> egui::Rect {
    let tile_width = tilemap.tile_size.width as f32;
    let tile_height = tilemap.tile_size.height as f32;
    let scale_x = item.transform.scale.x.abs().max(0.05);
    let scale_y = item.transform.scale.y.abs().max(0.05);
    let origin = item.transform.position;
    let map_size = egui::vec2(
        tilemap.map_size.width as f32 * tile_width * scale_x,
        tilemap.map_size.height as f32 * tile_height * scale_y,
    );
    let map_center = world_to_screen(Vec2::new(
        origin.x + map_size.x / 2.0,
        origin.y + map_size.y / 2.0,
    ));
    egui::Rect::from_center_size(map_center, map_size * viewport_zoom)
}

fn node_hit_rect(
    item: &NodeView,
    world_to_screen: &dyn Fn(Vec2) -> egui::Pos2,
    viewport_zoom: f32,
    texture_size: &mut impl FnMut(&str) -> Option<egui::Vec2>,
) -> egui::Rect {
    let center = world_to_screen(item.transform.position);
    let size = if let Some(sprite_path) = item.sprite_path.as_deref() {
        texture_size(sprite_path)
            .map(|size| {
                egui::vec2(
                    (size.x * item.transform.scale.x.abs().max(0.1) * viewport_zoom)
                        .clamp(8.0, 1024.0),
                    (size.y * item.transform.scale.y.abs().max(0.1) * viewport_zoom)
                        .clamp(8.0, 1024.0),
                )
            })
            .unwrap_or_else(|| {
                let marker_size = (42.0 * viewport_zoom).clamp(18.0, 84.0);
                egui::vec2(marker_size, marker_size)
            })
    } else {
        let marker_size = (34.0 * viewport_zoom).clamp(18.0, 72.0);
        egui::vec2(marker_size, marker_size)
    };

    egui::Rect::from_center_size(center, size)
}

fn resize_handle_rects(rect: egui::Rect) -> [(ResizeHandle, egui::Rect); 4] {
    [
        (
            ResizeHandle::TOP_LEFT,
            egui::Rect::from_center_size(rect.left_top(), egui::vec2(12.0, 12.0)),
        ),
        (
            ResizeHandle::TOP_RIGHT,
            egui::Rect::from_center_size(rect.right_top(), egui::vec2(12.0, 12.0)),
        ),
        (
            ResizeHandle::BOTTOM_LEFT,
            egui::Rect::from_center_size(rect.left_bottom(), egui::vec2(12.0, 12.0)),
        ),
        (
            ResizeHandle::BOTTOM_RIGHT,
            egui::Rect::from_center_size(rect.right_bottom(), egui::vec2(12.0, 12.0)),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crab2d_scene::{EntityId, TileSize, TilemapComponent, TilemapSize};

    #[test]
    fn hit_targets_preserve_draw_order_for_overlapping_nodes() {
        let first = node(EntityId::from_raw(1), Vec2::ZERO, None, None);
        let second = node(EntityId::from_raw(2), Vec2::ZERO, None, None);
        let items = vec![first, second];

        let targets = build_viewport_hit_targets(&items, &identity_world_to_screen, 1.0, |_| None);

        let hit = hit_target_at(&targets, egui::pos2(0.0, 0.0), 0.0).expect("target");
        assert_eq!(hit.entity, EntityId::from_raw(2));
    }

    #[test]
    fn sprite_hit_rect_uses_loaded_texture_size_and_scale() {
        let item = node(
            EntityId::from_raw(7),
            Vec2::new(10.0, -5.0),
            Some("hero.png"),
            None,
        )
        .with_scale(Vec2::new(2.0, 0.5));

        let targets = build_viewport_hit_targets(&[item], &identity_world_to_screen, 3.0, |path| {
            (path == "hero.png").then_some(egui::vec2(16.0, 32.0))
        });

        let rect = targets[0].rect;
        assert_eq!(rect.center(), egui::pos2(10.0, -5.0));
        assert_eq!(rect.size(), egui::vec2(96.0, 48.0));
    }

    #[test]
    fn selected_hit_rect_returns_topmost_target_for_entity() {
        let tilemap = TilemapComponent::new(TilemapSize::new(2, 1), TileSize::new(16, 16))
            .expect("tilemap should be valid");
        let item = node(EntityId::from_raw(3), Vec2::ZERO, None, Some(tilemap));

        let targets = build_viewport_hit_targets(&[item], &identity_world_to_screen, 1.0, |_| None);

        assert_eq!(targets.len(), 2);
        assert_eq!(
            selected_hit_rect(&targets, Some(EntityId::from_raw(3))),
            Some(targets[1].rect)
        );
    }

    fn identity_world_to_screen(position: Vec2) -> egui::Pos2 {
        egui::pos2(position.x, position.y)
    }

    fn node(
        id: EntityId,
        position: Vec2,
        sprite_path: Option<&str>,
        tilemap: Option<TilemapComponent>,
    ) -> NodeView {
        NodeView {
            id,
            name: "Node".to_owned(),
            transform: Transform2D::from_position(position),
            sprite_path: sprite_path.map(str::to_owned),
            camera: None,
            tilemap,
            collider: None,
            trigger: false,
        }
    }

    trait TestNodeViewExt {
        fn with_scale(self, scale: Vec2) -> Self;
    }

    impl TestNodeViewExt for NodeView {
        fn with_scale(mut self, scale: Vec2) -> Self {
            self.transform.scale = scale;
            self
        }
    }
}
