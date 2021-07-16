use super::WorldGeneratorType;
use crate::{
    dungeon_crawl::{Initiative, Player, TurnState},
    world_map::{Array2D, GridPosition, TileFactory, WorldMap},
    AppState,
};
use bevy::prelude::*;
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

const MAP_SIZE: i32 = 40;
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
    let mut tile_map;
    let mut max_fill_number;
    let mut max_fill_count;
    loop {
        tile_map = get_random_map();
        cellular_automata_steps(&mut tile_map, ITERATIONS);

        let mut current_fill_number = 1;
        max_fill_number = 0;
        max_fill_count = 0;
        for x in 2..MAP_SIZE - 2 {
            for y in 2..MAP_SIZE - 2 {
                if tile_map[[x, y]] == TileType::Alive(0) {
                    let count = flood_fill_from(&mut tile_map, (x, y), current_fill_number);

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

    let mut spawned_player = false;
    let mut entities = Array2D::<Vec<Entity>>::with_size(MAP_SIZE - 2, MAP_SIZE - 2);

    let tile_factory = TileFactory::new(&asset_server, &mut materials);
    for x in 1..MAP_SIZE - 1 {
        for y in 1..MAP_SIZE - 1 {
            let mut tile = vec![];

            if tile_map[[x, y]] == TileType::Alive(max_fill_number) {
                tile.push(tile_factory.floor(&mut commands, x - 1, y - 1));

                // Spawn player on the first top-left floor
                if !spawned_player {
                    spawned_player = true;
                    tile.push(
                        commands
                            .spawn_bundle(SpriteBundle {
                                material: materials.add(ColorMaterial {
                                    texture: Some(asset_server.load("hooded-figure.png")),
                                    color: Color::hex("EDEDED").unwrap(),
                                }),
                                transform: Transform::from_xyz(0.0, 0.0, 1.0),
                                ..Default::default()
                            })
                            .insert_bundle((
                                Player,
                                Initiative,
                                GridPosition { x: x - 1, y: y - 1 },
                            ))
                            .id(),
                    );
                }
            } else {
                // Show wall only if it's adjencent to a floor
                for i in -1..=1i32 {
                    for j in -1..=1i32 {
                        if tile_map[[x + i, y + j]] == TileType::Alive(max_fill_number) {
                            tile.push(tile_factory.wall(&mut commands, x - 1, y - 1));
                            break;
                        }
                    }
                }
            };

            entities[[x - 1, y - 1]] = tile;
        }
    }

    commands.insert_resource(WorldMap {
        entities,
        tile_factory,
        tiles: Array2D::with_size(MAP_SIZE - 2, MAP_SIZE - 2),
    });

    app_state
        .set(AppState::DungeonCrawl(TurnState::NewTurn))
        .unwrap();
}

fn get_random_map() -> Array2D<TileType> {
    let mut map = Array2D::<TileType>::with_elem(MAP_SIZE, MAP_SIZE, TileType::Dead);

    for x in 2..MAP_SIZE - 2 {
        for y in 2..MAP_SIZE - 2 {
            if random::<f32>() < ALIVE_SPAWN_CHANCE {
                map[[x, y]] = TileType::Alive(0);
            }
        }
    }

    map
}

fn cellular_automata_steps(map: &mut Array2D<TileType>, iterations: u32) {
    let mut map2 = Array2D::<TileType>::with_elem(MAP_SIZE, MAP_SIZE, TileType::Dead);

    for _ in 0..iterations {
        for x in 2..MAP_SIZE - 2 {
            for y in 2..MAP_SIZE - 2 {
                let mut neighbours = 0;
                for i in -1..=1i32 {
                    for j in -1..=1i32 {
                        if i == 0 && j == 0 {
                            continue;
                        }
                        if let TileType::Alive(_) = map[[x + i, y + j]] {
                            neighbours += 1;
                        }
                    }
                }

                if map[[x, y]] == TileType::Dead {
                    if neighbours > 4 {
                        map2[[x, y]] = TileType::Alive(0);
                    } else {
                        map2[[x, y]] = TileType::Dead;
                    }
                } else {
                    if neighbours < 3 {
                        map2[[x, y]] = TileType::Dead;
                    } else {
                        map2[[x, y]] = TileType::Alive(0);
                    }
                }
            }
        }
        mem::swap(map, &mut map2);
    }
    mem::swap(map, &mut map2);
}

fn flood_fill_from(map: &mut Array2D<TileType>, pos: (i32, i32), fill: usize) -> u32 {
    let mut tiles = vec![];
    if matches!(map[pos], TileType::Alive(_)) {
        tiles.push(pos);
    }

    let mut count = 0;

    while !tiles.is_empty() {
        let (x, y) = tiles.pop().unwrap();
        map[[x, y]] = TileType::Alive(fill);
        count += 1;

        for i in -1..=1i32 {
            for j in -1..=1i32 {
                if i != 0 && j != 0 {
                    continue;
                }
                let new = (x + i, y + j);
                if let TileType::Alive(f) = map[new] {
                    if f != fill {
                        tiles.push(new);
                    }
                }
            }
        }
    }

    count
}
