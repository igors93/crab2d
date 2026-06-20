use crab2d_core::{Engine, ProjectInfo};
use crab2d_scene::{
    Camera2DComponent, SceneError, SpriteComponent, TagComponent, TileCell, TileSize,
    TilemapComponent, TilemapSize, TilesetRef, Transform2D, Vec2,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectBootstrap {
    pub project_name: String,
    pub camera_name: String,
    pub player_name: String,
    pub world_root_name: String,
}

impl ProjectBootstrap {
    pub fn empty_project(project_name: impl Into<String>) -> Self {
        Self {
            project_name: project_name.into(),
            camera_name: "Camera2D".to_string(),
            player_name: "Player".to_string(),
            world_root_name: "WorldRoot".to_string(),
        }
    }

    pub fn apply(self, engine: &mut Engine) -> Result<(), SceneError> {
        engine.open_project(ProjectInfo::new(self.project_name));

        let camera = engine.active_scene.spawn_node(self.camera_name);
        engine
            .active_scene
            .add_camera(camera, Camera2DComponent::default())?;

        let player = engine.active_scene.spawn_node_with_transform(
            self.player_name,
            Transform2D::from_position(Vec2::new(160.0, 128.0)),
        )?;
        engine
            .active_scene
            .add_tag(player, TagComponent::new("player"))?;
        engine
            .active_scene
            .add_sprite(player, SpriteComponent::new("sprites/player.png"))?;

        let world_root = engine.active_scene.spawn_node(self.world_root_name);
        engine
            .active_scene
            .add_tag(world_root, TagComponent::new("world"))?;
        engine.active_scene.add_tilemap(
            world_root,
            starter_tilemap().map_err(SceneError::InvalidTilemap)?,
        )?;

        Ok(())
    }
}

fn starter_tilemap() -> Result<TilemapComponent, crab2d_scene::TilemapError> {
    let mut tilemap = TilemapComponent::new(TilemapSize::new(18, 12), TileSize::new(32, 32))?
        .with_tileset(TilesetRef::new("tilesets/grass_tileset.png", 4, 4))?;

    for y in 0..tilemap.map_size.height {
        for x in 0..tilemap.map_size.width {
            let border = x == 0
                || y == 0
                || x == tilemap.map_size.width - 1
                || y == tilemap.map_size.height - 1;
            let road = y == tilemap.map_size.height / 2 || x == tilemap.map_size.width / 2;
            let tile_index = if border {
                3
            } else if road {
                2
            } else if (x + y) % 5 == 0 {
                1
            } else {
                0
            };
            tilemap.set_tile("Ground", x, y, Some(TileCell::new(tile_index)))?;
        }
    }

    Ok(tilemap)
}

#[cfg(test)]
mod tests {
    use crab2d_core::{Engine, EngineConfig};

    use crate::ProjectBootstrap;

    #[test]
    fn starter_project_attaches_default_components_and_tilemap() {
        let mut engine = Engine::new(EngineConfig::new("Crab2D Test"));

        ProjectBootstrap::empty_project("Test Project")
            .apply(&mut engine)
            .expect("bootstrap should succeed");

        let player = engine
            .active_scene
            .nodes()
            .iter()
            .find(|node| node.name == "Player")
            .expect("player node should exist")
            .id;
        let world = engine
            .active_scene
            .nodes()
            .iter()
            .find(|node| node.name == "WorldRoot")
            .expect("world root node should exist")
            .id;

        assert!(engine.active_scene.sprite(player).is_some());
        assert!(engine.active_scene.tilemap(world).is_some());
        assert_eq!(
            engine
                .active_scene
                .tilemap(world)
                .expect("tilemap")
                .tile("Ground", 0, 0)
                .expect("tile read")
                .expect("tile exists")
                .tile_index,
            3
        );
    }
}
