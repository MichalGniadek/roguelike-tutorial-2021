use crate::{
    world_generation::WorldGeneratorType,
    world_map::{BlocksMovement, Grid, GridPosition, WorldMap},
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
            SystemSet::on_update(AppState::DungeonCrawl(TurnState::DuringTurn))
                .with_system(player_control.system()),
        );
    }
}

pub struct Player;
pub struct Initiative;

fn update_world_map(mut world: ResMut<WorldMap>, m: Query<&BlocksMovement>) {
    for x in 0..world.world_size.x {
        for y in 0..world.world_size.y {
            world.movement_blocked[[x, y]] = world.entities[[x, y]]
                .iter()
                .any(|e| matches!(m.get(*e), Ok(&BlocksMovement)));
        }
    }
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

    if *position != new_pos && !world.movement_blocked[new_pos] {
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
