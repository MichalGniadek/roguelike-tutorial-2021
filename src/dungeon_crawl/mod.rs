use crate::{
    world_generation::WorldGeneratorType,
    world_map::{Grid, GridPosition, Tile, WorldMap},
    AppState,
};
use bevy::{math::ivec2, prelude::*, render::camera::Camera};

pub struct DungeonCrawlPlugin;
impl Plugin for DungeonCrawlPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(Grid {
            cell_size: IVec2::new(512, 512),
        })
        .add_startup_system(
            (|mut commands: Commands| {
                let mut orto = OrthographicCameraBundle::new_2d();
                orto.orthographic_projection.scale = 8.0;
                commands
                    .spawn_bundle(orto)
                    .insert(GridPosition { x: 0, y: 0 });
            })
            .system(),
        )
        .add_system_set(
            SystemSet::on_enter(AppState::DungeonCrawl).with_system(spawn_player.system()),
        )
        .add_system_set(
            SystemSet::on_update(AppState::DungeonCrawl)
                .label("display")
                .with_system(camera_position.system().label("camera"))
                .with_system(update_position.system().after("camera")),
        )
        .add_system_set(
            SystemSet::on_update(AppState::DungeonCrawl)
                .after("display")
                .with_system(player_control.system()),
        )
        .add_system_set(
            SystemSet::on_exit(AppState::DungeonCrawl)
                .after("display")
                .with_system(cleanup_play.system()),
        );
    }
}

struct Player;
struct Initiative;

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    world: Res<WorldMap>,
    tile_query: Query<&Tile>,
) {
    let mut player_pos = ivec2(0, 0);
    'finished: for x in 0..world.world_size.x {
        for y in 0..world.world_size.y {
            match world.tiles[x as usize][y as usize] {
                Some(e) => {
                    if tile_query.get(e).unwrap().walkable {
                        player_pos = ivec2(x, y);
                        break 'finished;
                    }
                }
                None => continue,
            }
        }
    }

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
            GridPosition {
                x: player_pos.x,
                y: player_pos.y,
            },
        ));

    // commands
    //     .spawn_bundle(SpriteBundle {
    //         material: materials.add(ColorMaterial {
    //             texture: Some(asset_server.load("orc-head.png")),
    //             color: Color::hex("DA0037").unwrap(),
    //         }),
    //         transform: Transform::from_xyz(0.0, 0.0, 1.0),
    //         ..Default::default()
    //     })
    //     .insert(GridPosition { x: 7, y: 7 });
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
    tile_query: Query<&Tile>,
    world: Res<WorldMap>,
    keys: Res<Input<KeyCode>>,
    mut temp_app_state: ResMut<State<AppState>>,
) {
    let mut position = query.single_mut().unwrap();
    let mut new_pos = position.clone();

    if keys.is_changed() {
        match keys.get_just_pressed().next() {
            Some(KeyCode::Up | KeyCode::W) => new_pos.y += 1,
            Some(KeyCode::Down | KeyCode::S) => new_pos.y -= 1,
            Some(KeyCode::Left | KeyCode::A) => new_pos.x -= 1,
            Some(KeyCode::Right | KeyCode::D) => new_pos.x += 1,
            Some(KeyCode::R) => temp_app_state
                .set(AppState::WorldGeneration(
                    WorldGeneratorType::CellularAutomata,
                ))
                .unwrap(),
            _ => {}
        }
    }

    let tile = world.tiles[new_pos.x as usize][new_pos.y as usize].unwrap();
    let tile = tile_query.get(tile).unwrap();
    if tile.walkable {
        *position = new_pos;
    }
}

fn camera_position(
    mut query: QuerySet<(
        Query<&GridPosition, With<Player>>,
        Query<&mut GridPosition, With<Camera>>,
    )>,
) {
    let position = query.q0_mut().single_mut().unwrap().clone();
    let mut camera = query.q1_mut().single_mut().unwrap();
    *camera = position;
}

fn cleanup_play(
    tile_query: Query<(Entity, &Tile)>,
    player_query: Query<(Entity, &Player)>,
    mut commands: Commands,
) {
    for (e, _) in tile_query.iter() {
        commands.entity(e).despawn();
    }

    for (e, _) in player_query.iter() {
        commands.entity(e).despawn();
    }

    commands.remove_resource::<WorldMap>();
}