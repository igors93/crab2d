use crab2d_scene::{
    Collider2DComponent, PlayerControllerComponent, Scene, TileCell, TileSize, TilemapComponent,
    TilemapSize, Transform2D, Vec2, Velocity2DComponent,
};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::GenerationSettings;

const TILE_GRASS: u32 = 0;
const TILE_GRASS2: u32 = 1;
const TILE_DIRT: u32 = 2;
const TILE_WALL: u32 = 3;
const TILE_WATER: u32 = 4;
const TILE_TREE: u32 = 6;
const TILE_FLOOR: u32 = 7;

pub fn generate_starter_village(settings: &GenerationSettings) -> Scene {
    let mut scene = Scene::new(&settings.scene_name);
    let width = settings.map_width;
    let height = settings.map_height;
    let tile_size = settings.tile_size;
    let seed = settings.seed.unwrap_or_else(|| rand::thread_rng().gen());
    let mut rng = StdRng::seed_from_u64(seed);

    // --- Generate terrain grid using cellular automata ---
    let mut grid = vec![TILE_GRASS; (width * height) as usize];

    // Random noise pass
    for cell in grid.iter_mut() {
        *cell = if rng.gen::<f32>() < 0.1 {
            TILE_GRASS2
        } else if rng.gen::<f32>() < 0.05 {
            TILE_TREE
        } else {
            TILE_GRASS
        };
    }

    // Add a river (water strip)
    let river_x = width / 3;
    for y in 0..height {
        let wobble = (rng.gen::<i32>() % 2).unsigned_abs();
        let rx = (river_x + wobble).min(width - 1);
        grid[(y * width + rx) as usize] = TILE_WATER;
        if rx + 1 < width {
            grid[(y * width + rx + 1) as usize] = TILE_WATER;
        }
    }

    // Add village houses (3x3 rectangles with walls and floor)
    let num_houses = 3 + rng.gen::<u32>() % 3;
    for _ in 0..num_houses {
        let hx = river_x + 5 + rng.gen::<u32>() % (width / 3);
        let hy = 3 + rng.gen::<u32>() % (height.saturating_sub(10).max(1));
        place_house(&mut grid, width, height, hx, hy);
    }

    // Add dirt paths from houses toward center
    let path_y = height / 2;
    for x in river_x..width {
        grid[(path_y * width + x) as usize] = TILE_DIRT;
        if path_y + 1 < height {
            grid[((path_y + 1) * width + x) as usize] = TILE_DIRT;
        }
    }

    // --- Build tilemap component ---
    let Ok(mut tilemap) = TilemapComponent::new(
        TilemapSize::new(width, height),
        TileSize::new(tile_size, tile_size),
    ) else {
        return scene;
    };
    // Walls are solid
    tilemap.collision.set_solid(TILE_WALL, true);
    tilemap.collision.set_solid(TILE_WATER, true);
    tilemap.collision.set_solid(TILE_TREE, true);

    // Fill layer from grid
    for y in 0..height {
        for x in 0..width {
            let tile_index = grid[(y * width + x) as usize];
            let _ = tilemap.set_tile("Ground", x, y, Some(TileCell::new(tile_index)));
        }
    }

    // Spawn tilemap node
    let world_node = scene.spawn_node("WorldMap");
    let _ = scene.add_tilemap(world_node, tilemap);

    // Spawn a player node at center
    let player_start = Vec2::new(
        (width as f32 * tile_size as f32) * 0.66,
        (height as f32 * tile_size as f32) * 0.5,
    );
    let Ok(player) =
        scene.spawn_node_with_transform("Player", Transform2D::from_position(player_start))
    else {
        return scene;
    };
    let _ = scene.add_velocity(player, Velocity2DComponent::default());
    let _ = scene.add_collider(
        player,
        Collider2DComponent::rectangle(Vec2::new(16.0, 16.0)),
    );
    let _ = scene.add_player_controller(player, PlayerControllerComponent::new(120.0));

    scene
}

fn place_house(grid: &mut [u32], width: u32, height: u32, x: u32, y: u32) {
    let w = 6u32;
    let h = 5u32;
    for ry in 0..h {
        for rx in 0..w {
            let gx = x + rx;
            let gy = y + ry;
            if gx >= width || gy >= height {
                continue;
            }
            let is_wall = rx == 0 || rx == w - 1 || ry == 0 || ry == h - 1;
            grid[(gy * width + gx) as usize] = if is_wall { TILE_WALL } else { TILE_FLOOR };
        }
    }
}

// Struct wrapper implementing the WorldGenerator trait
#[derive(Debug, Default)]
pub struct StarterVillageGenerator;

impl crate::WorldGenerator for StarterVillageGenerator {
    fn generate_scene(&self, settings: &GenerationSettings) -> Scene {
        generate_starter_village(settings)
    }
}
