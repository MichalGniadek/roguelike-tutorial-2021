use super::WorldGeneratorType;
use crate::{
    world_map::{TileFactory, WorldMap},
    AppState,
};
use bevy::{math::ivec2, prelude::*};
use rand::random;
use std::mem;

pub struct CellularAutomataPlugin;
impl Plugin for CellularAutomataPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::on_enter(AppState::WorldGeneration(
                WorldGeneratorType::CellularAutomata,
            ))
            .with_system(cellular_automata.system()),
        );
    }
}

const MAP_SIZE: usize = 40;
const ALIVE_SPAWN_CHANCE: f32 = 0.45;
const ITERATIONS: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TileType {
    Alive(usize),
    Dead,
}

fn cellular_automata(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut app_state: ResMut<State<AppState>>,
) {
    let mut map;
    let mut max_fill_number;
    let mut max_fill_count;
    loop {
        map = get_random_map();
        cellular_automata_steps(&mut map, ITERATIONS);

        let mut current_fill_number = 1;
        max_fill_number = 0;
        max_fill_count = 0;
        for x in 2..MAP_SIZE - 2 {
            for y in 2..MAP_SIZE - 2 {
                if map[x][y] == TileType::Alive(0) {
                    let count = flood_fill_from(&mut map, (x, y), current_fill_number);

                    if max_fill_count < count {
                        max_fill_count = count;
                        max_fill_number = current_fill_number;
                    }

                    current_fill_number += 1;
                }
            }
        }

        if max_fill_count >= 400 && max_fill_count <= 600 {
            break;
        }
    }

    let tile_factory = TileFactory::new(&asset_server, &mut materials);
    let mut tiles = vec![];
    for x in 1..MAP_SIZE - 1 {
        let mut column = vec![];
        for y in 1..MAP_SIZE - 1 {
            if map[x][y] == TileType::Alive(max_fill_number) {
                column.push(Some(
                    commands
                        .spawn_bundle(tile_factory.floor(x as i32 - 1, y as i32 - 1))
                        .id(),
                ));
            } else {
                let mut adjacent_floor = false;
                for i in -1..=1i32 {
                    for j in -1..=1i32 {
                        if map[(x as i32 + i) as usize][(y as i32 + j) as usize]
                            == TileType::Alive(max_fill_number)
                        {
                            adjacent_floor = true;
                        }
                    }
                }
                if adjacent_floor {
                    column.push(Some(
                        commands
                            .spawn_bundle(tile_factory.wall(x as i32 - 1, y as i32 - 1))
                            .id(),
                    ));
                } else {
                    column.push(None)
                }
            };
        }
        tiles.push(column);
    }

    commands.insert_resource(WorldMap {
        world_size: ivec2(MAP_SIZE as i32 - 2, MAP_SIZE as i32 - 2),
        tiles,
        tile_factory,
    });
    app_state.set(AppState::DungeonCrawl).unwrap();
}

fn get_random_map() -> Vec<Vec<TileType>> {
    let mut map = vec![vec![TileType::Dead; MAP_SIZE]; MAP_SIZE];

    for x in 2..MAP_SIZE - 2 {
        for y in 2..MAP_SIZE - 2 {
            if random::<f32>() < ALIVE_SPAWN_CHANCE {
                map[x][y] = TileType::Alive(0);
            } else {
                map[x][y] = TileType::Dead;
            }
        }
    }

    map
}

fn cellular_automata_steps(map: &mut Vec<Vec<TileType>>, iterations: u32) {
    let mut map2 = vec![vec![TileType::Dead; MAP_SIZE]; MAP_SIZE];

    for _ in 0..iterations {
        for x in 2..MAP_SIZE - 2 {
            for y in 2..MAP_SIZE - 2 {
                let mut neighbours = 0;
                for i in -1..=1i32 {
                    for j in -1..=1i32 {
                        if i == 0 && j == 0 {
                            continue;
                        }
                        if let TileType::Alive(_) =
                            map[(x as i32 + i) as usize][(y as i32 + j) as usize]
                        {
                            neighbours += 1;
                        }
                    }
                }

                if map[x][y] == TileType::Dead {
                    if neighbours > 4 {
                        map2[x][y] = TileType::Alive(0);
                    } else {
                        map2[x][y] = TileType::Dead;
                    }
                } else {
                    if neighbours < 3 {
                        map2[x][y] = TileType::Dead;
                    } else {
                        map2[x][y] = TileType::Alive(0);
                    }
                }
            }
        }
        mem::swap(map, &mut map2);
    }
    mem::swap(map, &mut map2);
}

fn flood_fill_from(map: &mut Vec<Vec<TileType>>, pos: (usize, usize), fill: usize) -> u32 {
    let mut tiles = vec![];
    if matches!(map[pos.0][pos.1], TileType::Alive(_)) {
        tiles.push(pos);
    }

    let mut count = 0;

    while !tiles.is_empty() {
        let (x, y) = tiles.pop().unwrap();
        map[x][y] = TileType::Alive(fill);
        count += 1;

        for i in -1..=1i32 {
            for j in -1..=1i32 {
                if i != 0 && j != 0 {
                    continue;
                }
                let new_x = (x as i32 + i) as usize;
                let new_y = (y as i32 + j) as usize;
                if let TileType::Alive(f) = map[new_x][new_y] {
                    if f != fill {
                        tiles.push((new_x, new_y));
                    }
                }
            }
        }
    }

    count
}
