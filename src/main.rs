use bevy::prelude::*;

fn main() {
    App::build()
        .insert_resource(ClearColor(Color::hex("171717").unwrap()))
        .add_plugins(DefaultPlugins)
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .insert_resource(Grid {
            cell_size: IVec2::new(512, 512),
        })
        .add_startup_system(
            // Setup camera
            (|mut commands: Commands| {
                let mut orto = OrthographicCameraBundle::new_2d();
                orto.orthographic_projection.scale = 8.0;
                commands.spawn_bundle(orto);
            })
            .system(),
        )
        .add_startup_system(generate_world.system())
        .add_system(player_control.system())
        .add_system(update_position.system())
        .run();
}

struct Player;
struct Initiative;
struct Grid {
    cell_size: IVec2,
}
#[derive(Debug, Clone, Copy)]
struct GridPosition {
    x: i32,
    y: i32,
}

struct Tile {
    walkable: bool,
}

struct World {
    world_size: IVec2,
    tiles: Vec<Vec<Entity>>,
}

fn generate_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let wall_material = materials.add(ColorMaterial {
        texture: Some(asset_server.load("brick-wall.png")),
        color: Color::hex("444444").unwrap(),
    });

    let floor_material = materials.add(ColorMaterial {
        texture: Some(asset_server.load("square.png")),
        color: Color::hex("444444").unwrap(),
    });

    let world_size = IVec2::new(10, 10);
    let mut tiles = vec![];
    for x in 0..world_size.x {
        let mut column = vec![];
        for y in 0..world_size.y {
            if x == 0 || x == world_size.x - 1 || y == 0 || y == world_size.y - 1 {
                column.push(
                    commands
                        .spawn_bundle(SpriteBundle {
                            material: wall_material.clone(),
                            ..Default::default()
                        })
                        .insert(GridPosition { x, y })
                        .insert(Tile { walkable: false })
                        .id(),
                );
            } else {
                column.push(
                    commands
                        .spawn_bundle(SpriteBundle {
                            material: floor_material.clone(),
                            ..Default::default()
                        })
                        .insert(GridPosition { x, y })
                        .insert(Tile { walkable: true })
                        .id(),
                );
            }
        }
        tiles.push(column);
    }

    commands.insert_resource(World { world_size, tiles });

    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(ColorMaterial {
                texture: Some(asset_server.load("hooded-figure.png")),
                color: Color::hex("EDEDED").unwrap(),
            }),
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            ..Default::default()
        })
        .insert(Player)
        .insert(Initiative)
        .insert(GridPosition { x: 4, y: 4 });

    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(ColorMaterial {
                texture: Some(asset_server.load("orc-head.png")),
                color: Color::hex("DA0037").unwrap(),
            }),
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            ..Default::default()
        })
        .insert(GridPosition { x: 7, y: 7 });
}

fn update_position(
    mut query: Query<(&mut Transform, &GridPosition), Changed<GridPosition>>,
    grid: Res<Grid>,
    world: Res<World>,
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
    world: Res<World>,
    keys: Res<Input<KeyCode>>,
) {
    let mut position = query.single_mut().unwrap();
    let mut new_pos = position.clone();

    match keys.get_just_pressed().next() {
        Some(KeyCode::Up | KeyCode::W) => new_pos.y += 1,
        Some(KeyCode::Down | KeyCode::S) => new_pos.y -= 1,
        Some(KeyCode::Left | KeyCode::A) => new_pos.x -= 1,
        Some(KeyCode::Right | KeyCode::D) => new_pos.x += 1,
        _ => {}
    }

    let tile = world.tiles[new_pos.x as usize][new_pos.y as usize];
    let tile = tile_query.get(tile).unwrap();
    if tile.walkable {
        *position = new_pos;
    }
}
