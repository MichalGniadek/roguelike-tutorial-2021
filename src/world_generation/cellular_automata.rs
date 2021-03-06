use crate::{
    bundles::{EnemyBundle, ItemBundle, PlayerBundle},
    dungeon_crawl::{GameData, InitiativeOrder},
    world_map::{Array2D, GridPosition, TileFactory, WorldMap},
    AppState,
};
use bevy::prelude::*;
use rand::random;
use std::{collections::VecDeque, mem};

pub struct CellularAutomataPlugin;
impl Plugin for CellularAutomataPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::on_enter(AppState::WorldGeneration).with_system(cellular_automata.system()),
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
    data: Res<GameData>,
) {
    let target_size = data.floor_map_size();
    let (tile_map, mut zone_entities) = loop {
        let mut tile_map = get_random_map();
        cellular_automata_steps(&mut tile_map, ITERATIONS);

        let size = select_largest_cave(&mut tile_map);
        if size < target_size.0 || size > target_size.1 {
            continue;
        }

        let zone_count = split_into_zones(&mut tile_map);
        if zone_count < 5 {
            continue;
        }

        break (
            tile_map,
            get_zone_entities(
                &mut commands,
                &asset_server,
                &mut materials,
                &*data,
                zone_count,
            ),
        );
    };

    let mut stairs = GridPosition { x: 1, y: 1 };
    while tile_map[stairs] == TileType::Dead {
        stairs = GridPosition {
            x: (1 + rand::random::<u32>() % (MAP_SIZE as u32 - 3)) as i32,
            y: (1 + rand::random::<u32>() % (MAP_SIZE as u32 - 3)) as i32,
        };
    }
    let mut entities = Array2D::with_size(MAP_SIZE + 20, MAP_SIZE + 20);
    let tile_factory = TileFactory::new(&asset_server, &mut materials);
    for x in 1..MAP_SIZE - 1 {
        for y in 1..MAP_SIZE - 1 {
            let mut tile = vec![];

            if stairs.x == x && stairs.y == y {
                tile.push(tile_factory.stairs(&mut commands, x + 9, y + 9));
            } else if let TileType::Alive(zone) = tile_map[[x, y]] {
                tile.push(tile_factory.floor(&mut commands, x + 9, y + 9));

                // Zones start at 1 so we have to substract one
                if let Some(e) = zone_entities.get_mut(zone - 1) {
                    if let Some(e) = e.pop() {
                        commands
                            .entity(e)
                            .insert(GridPosition { x: x + 9, y: y + 9 });
                        tile.push(e);
                    }
                }
            } else {
                // Show wall only if it's adjencent to a floor
                'finish: for i in -1..=1i32 {
                    for j in -1..=1i32 {
                        if let TileType::Alive(_) = tile_map[[x + i, y + j]] {
                            tile.push(tile_factory.wall(&mut commands, x + 9, y + 9));
                            break 'finish;
                        }
                    }
                }
            };

            entities[[x + 9, y + 9]] = tile;
        }
    }

    // Despawn unused enemies
    for e in zone_entities.iter().flatten() {
        commands.entity(*e).despawn();
    }

    commands.insert_resource(WorldMap {
        entities,
        tile_factory,
        tiles: Array2D::with_size(MAP_SIZE + 20, MAP_SIZE + 20),
        stairs: GridPosition {
            x: stairs.x + 9,
            y: stairs.y + 9,
        },
    });
    commands.insert_resource(InitiativeOrder::default());
    app_state.set(AppState::DungeonCrawlEnter).unwrap();
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

fn flood_fill(
    map: &mut Array2D<TileType>,
    pos: (i32, i32),
    fill: TileType,
    distance: Option<u32>,
) -> u32 {
    let mut tiles = VecDeque::new();
    tiles.push_back((pos, 0));

    let mut count = 0;

    while !tiles.is_empty() {
        let ((x, y), dist) = tiles.pop_front().unwrap();

        if map[[x, y]] != TileType::Alive(0) {
            continue;
        }

        map[[x, y]] = fill;
        count += 1;

        for i in -1..=1i32 {
            for j in -1..=1i32 {
                // No diagonals or the same tile
                if (i != 0 && j != 0) || (i == 0 && j == 0) {
                    continue;
                }

                let new = (x + i, y + j);

                if map[new] == TileType::Alive(0) {
                    if let Some(distance) = distance {
                        if distance > dist {
                            tiles.push_back((new, dist + 1));
                        }
                    } else {
                        tiles.push_back((new, dist + 1));
                    }
                }
            }
        }
    }

    count
}

fn select_largest_cave(tile_map: &mut Array2D<TileType>) -> u32 {
    let mut current_fill_number = 0;
    let mut max_fill_number = 0;
    let mut max_fill_count = 0;
    for x in 2..MAP_SIZE - 2 {
        for y in 2..MAP_SIZE - 2 {
            if tile_map[[x, y]] == TileType::Alive(0) {
                current_fill_number += 1;
                let count =
                    flood_fill(tile_map, (x, y), TileType::Alive(current_fill_number), None);

                if max_fill_count < count {
                    max_fill_count = count;
                    max_fill_number = current_fill_number;
                }
            }
        }
    }

    for x in 2..MAP_SIZE - 2 {
        for y in 2..MAP_SIZE - 2 {
            if let TileType::Alive(fill) = tile_map[[x, y]] {
                if fill == 0 {
                    continue;
                } else if fill == max_fill_number {
                    tile_map[[x, y]] = TileType::Alive(0);
                } else {
                    tile_map[[x, y]] = TileType::Dead;
                }
            }
        }
    }

    max_fill_count
}

fn split_into_zones(tile_map: &mut Array2D<TileType>) -> usize {
    let mut current_fill_number = 0;
    for x in 2..MAP_SIZE - 2 {
        for y in 2..MAP_SIZE - 2 {
            if tile_map[[x, y]] == TileType::Alive(0) {
                current_fill_number += 1;
                flood_fill(
                    tile_map,
                    (x, y),
                    TileType::Alive(current_fill_number),
                    Some(10),
                );
            }
        }
    }
    current_fill_number
}

fn get_zone_entities(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    data: &GameData,
    zone_count: usize,
) -> Vec<Vec<Entity>> {
    let mut entities = vec![vec![]; zone_count];
    entities[0].push(
        commands
            .spawn_bundle(PlayerBundle::new(asset_server, materials, data))
            .id(),
    );

    for _ in 0..data.floor_enemy_count() {
        let zone = (random::<usize>() % (zone_count - 1)) + 1;
        entities[zone].push(
            commands
                .spawn_bundle(EnemyBundle::orc(asset_server, materials))
                .id(),
        );
    }

    for _ in 0..data.floor_item_count() {
        let zone = random::<usize>() % (zone_count - 1) + 1;
        let item = data.floor_item();
        entities[zone].push(
            commands
                .spawn_bundle(ItemBundle::item(item, asset_server, materials))
                .id(),
        );
    }

    entities
}
