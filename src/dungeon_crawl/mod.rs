use crate::{
    world_map::{BlocksMovement, BlocksVision, Grid, GridPosition, Tile, TileFlags, WorldMap},
    AppState,
};
use bevy::{ecs::system::QuerySingleError, prelude::*, render::camera::Camera};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnState {
    NewTurn,
    DuringTurn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Wait,
    Move(Entity, GridPosition, GridPosition),
    Attack(Entity),
}

#[derive(Default)]
pub struct InitiativeOrder {
    order: Vec<Entity>,
    current: usize,
}

pub struct DungeonCrawlPlugin;
impl Plugin for DungeonCrawlPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<Action>().init_resource::<InitiativeOrder>();

        app.add_system_set(
            SystemSet::on_enter(AppState::DungeonCrawl(TurnState::NewTurn))
                .with_system(update_position.system().before("camera"))
                .with_system(camera_position.system().label("camera"))
                .with_system(update_world_map.system().label("update_world_map"))
                .with_system(handle_initiative.system())
                .with_system(player_fov.system().before("update_world_map"))
                .with_system(
                    (|mut app_state: ResMut<State<AppState>>| {
                        let _ = app_state.set(AppState::DungeonCrawl(TurnState::DuringTurn));
                    })
                    .system(),
                ),
        );

        app.add_system_set(
            SystemSet::on_update(AppState::DungeonCrawl(TurnState::DuringTurn))
                .with_system(player_control.system().before("actions"))
                .with_system(enemy_ai.system().before("actions"))
                .with_system(handle_actions.system().label("actions"))
                .with_system(end_turn.system().label("actions")),
        );
    }
}

pub struct Player;
pub struct Enemy;
pub struct Initiative;

fn update_world_map(
    mut world: ResMut<WorldMap>,
    m: Query<&BlocksMovement>,
    v: Query<&BlocksVision>,
) {
    let world_size = world.entities.size();
    for x in 0..world_size.x {
        for y in 0..world_size.y {
            world.tiles[[x, y]] &= TileFlags::EXPLORED;
        }
    }

    for x in 0..world_size.x {
        for y in 0..world_size.y {
            if world.entities[[x, y]]
                .iter()
                .any(|e| matches!(m.get(*e), Ok(&BlocksMovement)))
            {
                world.tiles[[x, y]] |= TileFlags::BLOCKS_MOVEMENT;
            }
        }
    }

    for x in 0..world_size.x {
        for y in 0..world_size.y {
            if world.entities[[x, y]]
                .iter()
                .any(|e| matches!(v.get(*e), Ok(&BlocksVision)))
            {
                world.tiles[[x, y]] |= TileFlags::BLOCKS_VISION;
            }
        }
    }
}

fn handle_initiative(
    mut order: ResMut<InitiativeOrder>,
    characters: Query<Entity, Or<(With<Player>, With<Enemy>)>>,
    mut commands: Commands,
) {
    if let Some(e) = order.order.get(order.current) {
        commands.entity(*e).remove::<Initiative>();
    }

    for c in characters.iter() {
        if !order.order.contains(&c) {
            order.order.push(c);
        }
    }

    order.current += 1;
    if order.order.len() > 0 {
        order.current %= order.order.len();
        commands
            .entity(order.order[order.current])
            .insert(Initiative);
    }
}

fn player_fov(
    player: Query<&GridPosition, With<Player>>,
    mut visible: Query<(&mut Visible, &GridPosition, Option<&Tile>)>,
    mut tiles: Query<(&mut Handle<ColorMaterial>, &GridPosition), With<Tile>>,
    mut world: ResMut<WorldMap>,
) {
    let position = *player.single().unwrap();

    for end in fov_circle(position.x, position.y, 4) {
        let mut previous = None;
        for (x, y) in line_drawing::Bresenham::new((position.x, position.y), end) {
            if let Some(&tile) = world.tiles.get(x, y) {
                // Don't go through diagonal walls.
                if let Some((prev_x, prev_y)) = previous {
                    if (world.tiles[[prev_x, y]] & world.tiles[[x, prev_y]])
                        .contains(TileFlags::BLOCKS_VISION)
                    {
                        break;
                    }
                }
                previous = Some((x, y));

                world.tiles[[x, y]] |= TileFlags::IN_VIEW;

                // Remove artifacts
                if !tile.contains(TileFlags::BLOCKS_VISION) {
                    // Different direction depending in which quadrant we are in.
                    for (i, j) in [
                        ((x - position.x).signum(), 0),
                        (0, (y - position.y).signum()),
                    ] {
                        if let Some(neigh) = world.tiles.get_mut(x + i, y + j) {
                            if neigh.contains(TileFlags::BLOCKS_VISION) {
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

    for x in 0..world.entities.size().x {
        for y in 0..world.entities.size().y {
            if world.tiles[[x, y]].contains(TileFlags::IN_VIEW) {
                world.tiles[[x, y]] |= TileFlags::EXPLORED;
            }
        }
    }

    for (mut visible, pos, tile) in visible.iter_mut() {
        if let Some(_) = tile {
            visible.is_visible = world.tiles[[pos.x, pos.y]].contains(TileFlags::EXPLORED);
        } else {
            visible.is_visible = world.tiles[[pos.x, pos.y]].contains(TileFlags::IN_VIEW);
        }
    }

    for (mut mat, pos) in tiles.iter_mut() {
        if world.tiles[[pos.x, pos.y]].contains(TileFlags::IN_VIEW) {
            if mat.id == world.tile_factory.explored_floor_material.id {
                *mat = world.tile_factory.visible_floor_material.clone();
            } else if mat.id == world.tile_factory.explored_wall_material.id {
                *mat = world.tile_factory.visible_wall_material.clone();
            }
        } else {
            if mat.id == world.tile_factory.visible_floor_material.id {
                *mat = world.tile_factory.explored_floor_material.clone();
            } else if mat.id == world.tile_factory.visible_wall_material.id {
                *mat = world.tile_factory.explored_wall_material.clone();
            }
        }
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
        points.push((x + off, y - r - 1));
        points.push((x - off, y - r - 1));
        points.push((x + r + 1, y + off));
        points.push((x - r - 1, y + off));
        points.push((x + r + 1, y - off));
        points.push((x - r - 1, y - off));
    }
    points
}

fn update_position(
    mut query: Query<(&mut Transform, &GridPosition), Changed<GridPosition>>,
    grid: Res<Grid>,
    world: Res<WorldMap>,
) {
    let offset_x = (world.entities.size().x as f32 - 1.0) * (grid.cell_size.x as f32) / 2.0;
    let offset_y = (world.entities.size().y as f32 - 1.0) * (grid.cell_size.y as f32) / 2.0;
    for (mut transform, grid_position) in query.iter_mut() {
        transform.translation.x = (grid_position.x * grid.cell_size.x) as f32 - offset_x;
        transform.translation.y = (grid_position.y * grid.cell_size.y) as f32 - offset_y;
    }
}

fn player_control(
    mut query: Query<(Entity, &mut GridPosition), (With<Player>, With<Initiative>)>,
    enemies: Query<(), With<Enemy>>,
    world: Res<WorldMap>,
    keys: Res<Input<KeyCode>>,
    mut actions: EventWriter<Action>,
) {
    let (player_entity, position) = match query.single_mut() {
        Ok((e, pos)) => (e, pos),
        Err(QuerySingleError::NoEntities(_)) => return,
        Err(QuerySingleError::MultipleEntities(_)) => panic!(),
    };
    let mut new_pos = position.clone();

    if keys.is_changed() {
        match keys.get_just_pressed().next() {
            Some(KeyCode::Up | KeyCode::W) => new_pos.y += 1,
            Some(KeyCode::Down | KeyCode::S) => new_pos.y -= 1,
            Some(KeyCode::Left | KeyCode::A) => new_pos.x -= 1,
            Some(KeyCode::Right | KeyCode::D) => new_pos.x += 1,
            _ => {}
        }
    }

    if *position != new_pos {
        if world.tiles[new_pos].contains(TileFlags::BLOCKS_MOVEMENT) {
            for entity in &world.entities[new_pos] {
                if let Ok(()) = enemies.get(*entity) {
                    actions.send(Action::Attack(*entity));
                }
            }
        } else {
            actions.send(Action::Move(player_entity, *position, new_pos));
        }
    }
}

fn enemy_ai(enemy: Query<(), (With<Enemy>, With<Initiative>)>, mut actions: EventWriter<Action>) {
    let _enemy = match enemy.single() {
        Ok(e) => e,
        Err(QuerySingleError::NoEntities(_)) => return,
        Err(QuerySingleError::MultipleEntities(_)) => panic!(),
    };
    actions.send(Action::Wait);
}

fn handle_actions(
    mut actions: EventReader<Action>,
    mut positions: Query<&mut GridPosition>,
    mut world: ResMut<WorldMap>,
    mut app_state: ResMut<State<AppState>>,
) {
    for a in actions.iter() {
        match a {
            Action::Wait => {
                println!("Gaaarh!");
            }
            Action::Move(entity, old_pos, new_pos) => {
                let i = world.entities[*old_pos]
                    .iter()
                    .position(|x| x == entity)
                    .unwrap();
                world.entities[*old_pos].swap_remove(i);
                world.entities[*new_pos].push(*entity);

                if let Ok(mut pos) = positions.get_mut(*entity) {
                    *pos = *new_pos;
                }
            }
            Action::Attack(_) => println!("attack"),
        }
    }
}

fn end_turn(mut actions: EventReader<Action>, mut app_state: ResMut<State<AppState>>) {
    if actions.iter().count() != 0 {
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
