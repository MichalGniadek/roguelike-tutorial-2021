mod fov;
mod setup;

use self::setup::InitiativeOrder;
use crate::{
    world_map::{GridPosition, TileFlags, WorldMap},
    AppState,
};
use bevy::{ecs::system::QuerySingleError, prelude::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnState {
    Setup,
    Turn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Wait(Entity),
    Move(Entity, GridPosition, GridPosition),
    Attack(Entity, Entity, i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Kill(Entity);

pub struct DungeonCrawlPlugin;
impl Plugin for DungeonCrawlPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<Action>()
            .add_event::<Kill>()
            .init_resource::<InitiativeOrder>();

        use fov::*;
        use setup::*;
        app.add_system_set(
            SystemSet::on_enter(AppState::DungeonCrawl(TurnState::Setup))
                .with_system(update_position.system().before("camera"))
                .with_system(camera_position.system().label("camera"))
                .with_system(update_world_map.system().label("update_world_map"))
                .with_system(handle_initiative.system())
                .with_system(player_fov.system().after("update_world_map"))
                .with_system(
                    (|mut app_state: ResMut<State<AppState>>| {
                        let _ = app_state.set(AppState::DungeonCrawl(TurnState::Turn));
                    })
                    .system(),
                ),
        );

        app.add_system_set(
            SystemSet::on_update(AppState::DungeonCrawl(TurnState::Turn))
                .before("actions")
                .with_system(player_control.system())
                .with_system(enemy_ai.system()),
        );

        app.add_system_set(
            SystemSet::on_update(AppState::DungeonCrawl(TurnState::Turn))
                .label("actions")
                .with_system(handle_actions.system()),
        );

        app.add_system_set(
            SystemSet::on_update(AppState::DungeonCrawl(TurnState::Turn))
                .after("actions")
                .with_system(handle_kills.system())
                .with_system(end_turn.system()),
        );
    }
}

pub struct Player;
pub struct EnemyAI;
pub struct Initiative;

pub struct Health(pub i32);

fn player_control(
    mut query: Query<(Entity, &mut GridPosition), (With<Player>, With<Initiative>)>,
    enemies: Query<(), With<Health>>,
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
            for &entity in &world.entities[new_pos] {
                if let Ok(()) = enemies.get(entity) {
                    actions.send(Action::Attack(player_entity, entity, 1));
                }
            }
        } else {
            actions.send(Action::Move(player_entity, *position, new_pos));
        }
    }
}

fn enemy_ai(
    enemy: Query<Entity, (With<EnemyAI>, With<Initiative>)>,
    mut actions: EventWriter<Action>,
) {
    let enemy = match enemy.single() {
        Ok(e) => e,
        Err(QuerySingleError::NoEntities(_)) => return,
        Err(QuerySingleError::MultipleEntities(_)) => panic!(),
    };
    actions.send(Action::Wait(enemy));
}

fn handle_actions(
    mut actions: EventReader<Action>,
    mut positions: Query<&mut GridPosition>,
    mut healthy: Query<&mut Health>,
    mut world: ResMut<WorldMap>,
    mut kills: EventWriter<Kill>,
) {
    for a in actions.iter() {
        match a {
            Action::Wait(_) => {}
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
            Action::Attack(_, attackee, damage) => {
                let health = &mut healthy.get_mut(*attackee).unwrap().0;
                *health -= damage;

                if *health == 0 {
                    kills.send(Kill(*attackee));
                }
            }
        }
    }
}

fn handle_kills(
    mut kills: EventReader<Kill>,
    mut commands: Commands,
    mut order: ResMut<InitiativeOrder>,
    mut positions: Query<&mut GridPosition>,
    mut world: ResMut<WorldMap>,
) {
    for entity in kills.iter().map(|k| k.0) {
        commands.entity(entity).despawn();

        let pos = positions.get_mut(entity).unwrap();
        let i = world.entities[*pos]
            .iter()
            .position(|x| *x == entity)
            .unwrap();
        world.entities[*pos].swap_remove(i);

        let i = order.0.iter().position(|x| *x == entity).unwrap();
        order.0.remove(i);
    }
}

fn end_turn(mut actions: EventReader<Action>, mut app_state: ResMut<State<AppState>>) {
    if actions.iter().count() != 0 {
        app_state
            .set(AppState::DungeonCrawl(TurnState::Setup))
            .unwrap();
    }
}
