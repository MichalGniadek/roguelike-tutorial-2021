use crate::{
    world_generation::WorldGeneratorType,
    world_map::{BlocksMovement, BlocksVision, Grid, GridPosition, TileFlags, WorldMap},
    AppState,
};
use bevy::{prelude::*, render::camera::Camera};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnState {
    NewTurn,
    DuringTurn,
}

pub struct DungeonCrawlPlugin;
impl Plugin for DungeonCrawlPlugin {
    fn build(&self, app: &mut AppBuilder) {
        // New turn
        app.add_system_set(
            SystemSet::on_enter(AppState::DungeonCrawl(TurnState::NewTurn))
                .with_system(update_position.system().before("camera"))
                .with_system(camera_position.system().label("camera"))
                .with_system(update_world_map.system())
                .with_system(
                    (|mut app_state: ResMut<State<AppState>>| {
                        app_state
                            .set(AppState::DungeonCrawl(TurnState::DuringTurn))
                            .unwrap();
                    })
                    .system(),
                ),
        );

        // During turn
        app.add_system_set(
            SystemSet::on_enter(AppState::DungeonCrawl(TurnState::DuringTurn))
                .with_system(player_fov.system()),
        );
        app.add_system_set(
            SystemSet::on_update(AppState::DungeonCrawl(TurnState::DuringTurn))
                .with_system(player_control.system()),
        );
    }
}

pub struct Player;
pub struct Initiative;

fn update_world_map(
    mut world: ResMut<WorldMap>,
    m: Query<&BlocksMovement>,
    v: Query<&BlocksVision>,
) {
    for x in 0..world.world_size.x {
        for y in 0..world.world_size.y {
            world.tiles[[x, y]] = TileFlags::empty();
        }
    }

    for x in 0..world.world_size.x {
        for y in 0..world.world_size.y {
            if world.entities[[x, y]]
                .iter()
                .any(|e| matches!(m.get(*e), Ok(&BlocksMovement)))
            {
                world.tiles[[x, y]] |= TileFlags::BLOCKS_MOVEMENT;
            }
        }
    }

    for x in 0..world.world_size.x {
        for y in 0..world.world_size.y {
            if world.entities[[x, y]]
                .iter()
                .any(|e| matches!(v.get(*e), Ok(&BlocksVision)))
            {
                world.tiles[[x, y]] |= TileFlags::BLOCKS_VISION;
            }
        }
    }
}

fn player_fov(
    player: Query<&GridPosition, (With<Player>, With<Initiative>)>,
    mut entities: Query<(&mut Visible, &GridPosition)>,
    mut world: ResMut<WorldMap>,
) {
    let position = *player.single().unwrap();

    for end in fov_circle(position.x, position.y, 4) {
        for (x, y) in line_drawing::Bresenham::new((position.x, position.y), end) {
            if let Some(&tile) = world.tiles.get(x, y) {
                world.tiles[[x, y]] |= TileFlags::IN_VIEW;

                // Remove artifacts
                if !tile.contains(TileFlags::BLOCKS_MOVEMENT) {
                    let dir = [
                        (if position.x < x { 1 } else { -1 }, 0),
                        (0, if position.y < y { 1 } else { -1 }),
                    ];
                    for (i, j) in dir {
                        if let Some(neigh) = world.tiles.get_mut(x + i, y + j) {
                            if neigh.contains(TileFlags::BLOCKS_MOVEMENT) {
                                *neigh |= TileFlags::IN_VIEW;
                            }
                        }
                    }
                }

                if tile.contains(TileFlags::BLOCKS_VISION) {
                    break;
                }
            }
        }
    }

    for (mut visible, pos) in entities.iter_mut() {
        visible.is_visible = world.tiles[[pos.x, pos.y]].contains(TileFlags::IN_VIEW);
    }
}

fn fov_circle(x: i32, y: i32, r: i32) -> Vec<(i32, i32)> {
    let mut points = vec![];
    for off in 0..=r {
        points.push((x + off, y + r));
        points.push((x - off, y + r));
        points.push((x + off, y - r));
        points.push((x - off, y - r));
        points.push((x + r, y + off));
        points.push((x - r, y + off));
        points.push((x + r, y - off));
        points.push((x - r, y - off));
    }
    for off in 0..=(r / 2) {
        points.push((x + off, y + r + 1));
        points.push((x - off, y + r + 1));
        points.push((x + off, y - r + 1));
        points.push((x - off, y - r + 1));
        points.push((x + r + 1, y + off));
        points.push((x - r + 1, y + off));
        points.push((x + r + 1, y - off));
        points.push((x - r + 1, y - off));
    }
    points
}

fn update_position(
    mut query: Query<(&mut Transform, &GridPosition), Changed<GridPosition>>,
    grid: Res<Grid>,
    world: Res<WorldMap>,
) {
    let offset_x = (world.world_size.x as f32 - 1.0) * (grid.cell_size.x as f32) / 2.0;
    let offset_y = (world.world_size.y as f32 - 1.0) * (grid.cell_size.y as f32) / 2.0;
    for (mut transform, grid_position) in query.iter_mut() {
        transform.translation.x = (grid_position.x * grid.cell_size.x) as f32 - offset_x;
        transform.translation.y = (grid_position.y * grid.cell_size.y) as f32 - offset_y;
    }
}

fn player_control(
    mut query: Query<&mut GridPosition, (With<Player>, With<Initiative>)>,
    world: Res<WorldMap>,
    keys: Res<Input<KeyCode>>,
    mut app_state: ResMut<State<AppState>>,
) {
    let mut position = query.single_mut().unwrap();
    let mut new_pos = position.clone();

    if keys.is_changed() {
        match keys.get_just_pressed().next() {
            Some(KeyCode::Up | KeyCode::W) => new_pos.y += 1,
            Some(KeyCode::Down | KeyCode::S) => new_pos.y -= 1,
            Some(KeyCode::Left | KeyCode::A) => new_pos.x -= 1,
            Some(KeyCode::Right | KeyCode::D) => new_pos.x += 1,
            Some(KeyCode::R) => app_state
                .set(AppState::WorldGeneration(
                    WorldGeneratorType::CellularAutomata,
                ))
                .unwrap(),
            _ => {}
        }
    }

    if *position != new_pos && !world.tiles[new_pos].contains(TileFlags::BLOCKS_MOVEMENT) {
        *position = new_pos;
        app_state
            .set(AppState::DungeonCrawl(TurnState::NewTurn))
            .unwrap();
    }
}

fn camera_position(
    mut query: QuerySet<(
        Query<&Transform, With<Player>>,
        Query<&mut Transform, With<Camera>>,
    )>,
) {
    let mut position = query.q0_mut().single_mut().unwrap().clone();
    let mut camera = query.q1_mut().single_mut().unwrap();
    position.translation.z = camera.translation.z;
    *camera = position;
}

// fn cleanup_play(query: Query<Entity, With<GridPosition>>, mut commands: Commands) {
//     for e in query.iter() {
//         commands.entity(e).despawn();
//     }
//     commands.remove_resource::<WorldMap>();
// }
