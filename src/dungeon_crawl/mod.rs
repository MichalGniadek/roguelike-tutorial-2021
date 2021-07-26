mod fov;
mod setup;

use crate::{
    world_map::{GridPosition, TileFlags, WorldMap},
    AppState,
};
use bevy::{ecs::system::QuerySingleError, prelude::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnState {
    Setup,
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

        use fov::*;
        use setup::*;
        app.add_system_set(
            SystemSet::on_enter(AppState::DungeonCrawl(TurnState::Setup))
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
            .set(AppState::DungeonCrawl(TurnState::Setup))
            .unwrap();
    }
}
