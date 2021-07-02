use crate::{AppState, GridPosition};
use bevy::{math::ivec2, prelude::*};
use rand::random;
use std::cmp::{max, min};

pub struct Tile {
    pub walkable: bool,
}

pub struct World {
    pub world_size: IVec2,
    pub tiles: Vec<Vec<Option<Entity>>>,
}

// Generation

#[derive(Bundle)]
struct TileBundle {
    #[bundle]
    sprite: SpriteBundle,
    position: GridPosition,
    tile: Tile,
}

struct TileFactory {
    wall_material: Handle<ColorMaterial>,
    floor_material: Handle<ColorMaterial>,
}

impl TileFactory {
    fn wall(&self, x: i32, y: i32) -> TileBundle {
        TileBundle {
            sprite: SpriteBundle {
                material: self.wall_material.clone(),
                ..Default::default()
            },
            position: GridPosition { x, y },
            tile: Tile { walkable: false },
        }
    }

    fn floor(&self, x: i32, y: i32) -> TileBundle {
        TileBundle {
            sprite: SpriteBundle {
                material: self.floor_material.clone(),
                ..Default::default()
            },
            position: GridPosition { x, y },
            tile: Tile { walkable: true },
        }
    }
}

pub struct WorldGeneration {
    pub tiles: Vec<Vec<bool>>,
    pub bounds: (IVec2, IVec2),
    pub floor_number: u32,
}

impl WorldGeneration {
    pub fn get(&self, pos: IVec2) -> Option<bool> {
        if pos.x > 0
            && (pos.x as usize) < self.tiles.len()
            && pos.y > 0
            && (pos.y as usize) < self.tiles[pos.x as usize].len()
        {
            Some(self.tiles[pos.x as usize][pos.y as usize])
        } else {
            None
        }
    }

    pub fn set(&mut self, pos: IVec2, b: bool) -> u32 {
        if pos.x > 0
            && (pos.x as usize) < self.tiles.len()
            && pos.y > 0
            && (pos.y as usize) < self.tiles[pos.x as usize].len()
        {
            self.bounds.0.x = min(self.bounds.0.x, pos.x);
            self.bounds.0.y = min(self.bounds.0.y, pos.y);

            self.bounds.1.x = max(self.bounds.1.x, pos.x + 1);
            self.bounds.1.y = max(self.bounds.1.y, pos.y + 1);

            if !self.tiles[pos.x as usize][pos.y as usize] && b {
                self.floor_number += 1;
            } else if self.tiles[pos.x as usize][pos.y as usize] && !b {
                self.floor_number -= 1;
            }

            self.tiles[pos.x as usize][pos.y as usize] = b;
        }
        self.floor_number
    }
}

#[derive(Debug, Clone, Copy)]
enum Life {
    Time(u32),
    Floors(u32),
}

pub struct Drunkard {
    position: IVec2,
    direction: u32,
    direction_change_chance: f32,
    life: Life,

    spawn_chance: f32,
    spawn_life: Life,
}

pub fn start_world_generation(mut commands: Commands) {
    commands.insert_resource(WorldGeneration {
        tiles: vec![vec![false; 100]; 100],
        bounds: (ivec2(i32::MAX, i32::MAX), ivec2(i32::MIN, i32::MIN)),
        floor_number: 0,
    });

    commands.spawn().insert(Drunkard {
        position: ivec2(50, 50),
        direction: 0,
        direction_change_chance: 0.50,
        life: Life::Time(5),

        spawn_chance: 1.0,
        spawn_life: Life::Floors(250),
    });
}

pub fn drunkard_walk(
    mut drunkards: Query<(Entity, &mut Drunkard)>,
    mut world: ResMut<WorldGeneration>,
    mut commands: Commands,
    mut app_state: ResMut<State<AppState>>,
) {
    for (e, mut drunkard) in drunkards.iter_mut() {
        let floors_number = world.set(drunkard.position, true);

        let dir = match drunkard.direction {
            0 => ivec2(0, 1),
            1 => ivec2(1, 0),
            2 => ivec2(0, -1),
            3 => ivec2(-1, 0),
            _ => unreachable!(),
        };
        drunkard.position += dir;

        if drunkard.direction_change_chance > random::<f32>() {
            drunkard.direction = (1 + drunkard.direction + (random::<u32>() % 3)) % 4;
        }

        if drunkard.spawn_chance > random::<f32>() {
            commands.spawn().insert(Drunkard {
                position: drunkard.position,
                direction: random::<u32>() % 4,
                direction_change_chance: drunkard.direction_change_chance,
                life: drunkard.spawn_life,

                spawn_chance: 0.0,
                spawn_life: Life::Time(1),
            });
        }

        match &mut drunkard.life {
            Life::Time(t) => {
                *t -= 1;
                if *t == 0 {
                    commands.entity(e).despawn();
                }
            }
            Life::Floors(f) => {
                if *f <= floors_number {
                    commands.entity(e).despawn();
                }
            }
        };
    }

    if drunkards.iter_mut().len() == 0 {
        app_state.set(AppState::Play).unwrap();
    }
}

pub fn finish_world_generation(
    mut world: ResMut<WorldGeneration>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Add border
    world.bounds.0 -= ivec2(1, 1);
    world.bounds.1 += ivec2(1, 1);

    let world_size = world.bounds.1 - world.bounds.0;

    let tile_factory = TileFactory {
        wall_material: materials.add(ColorMaterial {
            texture: Some(asset_server.load("brick-wall.png")),
            color: Color::hex("444444").unwrap(),
        }),
        floor_material: materials.add(ColorMaterial {
            texture: Some(asset_server.load("square.png")),
            color: Color::hex("444444").unwrap(),
        }),
    };

    let mut tiles = vec![];
    for x in 0..world_size.x {
        let mut column = vec![];
        for y in 0..world_size.y {
            if world
                .get(ivec2(x + world.bounds.0.x, y + world.bounds.0.y))
                .unwrap()
            {
                column.push(Some(commands.spawn_bundle(tile_factory.floor(x, y)).id()));
            } else {
                let mut adjacent_floor = false;
                for i in -1..=1 {
                    for j in -1..=1 {
                        adjacent_floor |= world
                            .get(ivec2(x + i + world.bounds.0.x, y + j + world.bounds.0.y))
                            .unwrap_or(false);
                    }
                }
                if adjacent_floor {
                    column.push(Some(commands.spawn_bundle(tile_factory.wall(x, y)).id()));
                } else {
                    column.push(None)
                }
            };
        }
        tiles.push(column);
    }

    commands.insert_resource(World { world_size, tiles });
}
