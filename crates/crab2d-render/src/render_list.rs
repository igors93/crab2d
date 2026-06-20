use crab2d_scene::{EntityId, Scene, Transform2D};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RenderList {
    pub camera: Option<CameraRenderCommand>,
    pub items: Vec<RenderItem>,
}

impl RenderList {
    pub fn from_scene(scene: &Scene) -> Self {
        let camera = scene.nodes().iter().find_map(|node| {
            scene.camera(node.id).map(|camera| CameraRenderCommand {
                entity: node.id,
                transform: node.transform,
                zoom: camera.zoom,
                clear_color: camera.clear_color,
            })
        });

        let mut items = Vec::new();
        for (entity, tilemap) in scene.tilemaps() {
            let Some(node) = scene.node(entity) else {
                continue;
            };
            items.push(RenderItem::Tilemap(TilemapRenderCommand {
                entity,
                transform: node.transform,
                map_width: tilemap.map_size.width,
                map_height: tilemap.map_size.height,
                tile_width: tilemap.tile_size.width,
                tile_height: tilemap.tile_size.height,
                visible_tiles: tilemap.visible_tiles().len(),
                tileset_path: tilemap
                    .tileset
                    .as_ref()
                    .map(|tileset| tileset.image_path.clone()),
            }));
        }

        let mut sprites = scene
            .sprites()
            .filter_map(|(entity, sprite)| {
                if !sprite.visible {
                    return None;
                }
                scene.node(entity).map(|node| {
                    RenderItem::Sprite(SpriteRenderCommand {
                        entity,
                        transform: node.transform,
                        sprite_path: sprite.sprite_path.clone(),
                        z_index: sprite.z_index,
                    })
                })
            })
            .collect::<Vec<_>>();

        sprites.sort_by_key(|item| match item {
            RenderItem::Sprite(sprite) => sprite.z_index,
            RenderItem::Tilemap(_) => 0,
        });
        items.extend(sprites);

        Self { camera, items }
    }

    pub fn sprite_count(&self) -> usize {
        self.items
            .iter()
            .filter(|item| matches!(item, RenderItem::Sprite(_)))
            .count()
    }

    pub fn tilemap_count(&self) -> usize {
        self.items
            .iter()
            .filter(|item| matches!(item, RenderItem::Tilemap(_)))
            .count()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RenderItem {
    Sprite(SpriteRenderCommand),
    Tilemap(TilemapRenderCommand),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpriteRenderCommand {
    pub entity: EntityId,
    pub transform: Transform2D,
    pub sprite_path: String,
    pub z_index: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TilemapRenderCommand {
    pub entity: EntityId,
    pub transform: Transform2D,
    pub map_width: u32,
    pub map_height: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    pub visible_tiles: usize,
    pub tileset_path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraRenderCommand {
    pub entity: EntityId,
    pub transform: Transform2D,
    pub zoom: f32,
    pub clear_color: [f32; 4],
}

#[cfg(test)]
mod tests {
    use crab2d_scene::{
        Camera2DComponent, Scene, SpriteComponent, TileCell, TileSize, TilemapComponent,
        TilemapSize, Transform2D, Vec2,
    };

    use crate::{RenderItem, RenderList};

    #[test]
    fn render_list_extracts_camera_sprites_and_tilemaps() {
        let mut scene = Scene::new("Render Test");
        let camera = scene
            .spawn_node_with_transform("Camera2D", Transform2D::from_position(Vec2::new(1.0, 2.0)))
            .expect("camera should spawn");
        let player = scene.spawn_node("Player");
        let world = scene.spawn_node("World");
        let mut tilemap =
            TilemapComponent::new(TilemapSize::new(2, 2), TileSize::new(16, 16)).expect("tilemap");
        tilemap
            .set_tile("Ground", 0, 0, Some(TileCell::new(1)))
            .expect("tile should set");

        scene
            .add_camera(camera, Camera2DComponent::new().with_zoom(2.0))
            .expect("camera should attach");
        scene
            .add_sprite(player, SpriteComponent::new("sprites/player.png"))
            .expect("sprite should attach");
        scene
            .add_tilemap(world, tilemap)
            .expect("tilemap should attach");

        let list = RenderList::from_scene(&scene);

        assert_eq!(list.camera.expect("camera").entity, camera);
        assert_eq!(list.sprite_count(), 1);
        assert_eq!(list.tilemap_count(), 1);
        assert!(matches!(list.items[0], RenderItem::Tilemap(_)));
        assert!(matches!(list.items[1], RenderItem::Sprite(_)));
    }
}
