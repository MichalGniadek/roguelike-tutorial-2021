mod fov;
mod setup;
mod ui;

use self::setup::InitiativeOrder;
use crate::{
    dungeon_crawl::ui::LogMessage,
    world_map::{GridPosition, TileFlags, WorldMap},
    AppState,
};
use bevy::{app::AppExit, ecs::system::QuerySingleError, prelude::*};

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
            .add_event::<LogMessage>()
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
        app.add_system_set(
            SystemSet::on_update(AppState::DungeonCrawl(TurnState::Turn))
                .with_system(ui::update_health.system())
                .with_system(ui::update_log.system())
                .with_system(ui::update_details.system()),
        );
    }
}

pub struct Player {
    pub inventory: [Option<Entity>; 5],
}
pub struct EnemyAI;
pub struct Initiative;
pub struct Name(pub String);
pub enum Item {
    HealthPotion(i32),
}

impl Name {
    pub fn capitalized(&self) -> String {
        let mut chars = self.0.chars();
        let first = chars.next().unwrap().to_uppercase();
        format!("{}{}", first.collect::<String>(), chars.collect::<String>())
    }
}

pub struct Health {
    pub current: i32,
    pub max: i32,
}

impl Health {
    pub fn new(current: i32, max: i32) -> Self {
        Health { current, max }
    }
}

fn player_control(
    query: Query<(Entity, &GridPosition), (With<Player>, With<Initiative>)>,
    enemies: Query<(), With<Health>>,
    world: Res<WorldMap>,
    keys: Res<Input<KeyCode>>,
    mut actions: EventWriter<Action>,
) {
    let (player_entity, position) = match query.single() {
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
    enemy: Query<(Entity, &GridPosition), (With<EnemyAI>, With<Initiative>)>,
    player: Query<(Entity, &GridPosition), With<Player>>,
    world: Res<WorldMap>,
    mut actions: EventWriter<Action>,
) {
    let (enemy, position) = match enemy.single() {
        Ok(e) => e,
        Err(QuerySingleError::NoEntities(_)) => return,
        Err(QuerySingleError::MultipleEntities(_)) => panic!(),
    };

    if world.tiles[*position].contains(TileFlags::IN_VIEW) {
        let (player, player_pos) = player.single().unwrap();
        if let Some((path, _)) = world.pathfind(*position, *player_pos) {
            if path[1] == *player_pos {
                actions.send(Action::Attack(enemy, player, 1));
            } else if !world.tiles[path[1]].contains(TileFlags::BLOCKS_MOVEMENT) {
                actions.send(Action::Move(enemy, *position, path[1]));
            } else {
                actions.send(Action::Wait(enemy));
            }
        } else {
            actions.send(Action::Wait(enemy));
        }
    } else {
        actions.send(Action::Wait(enemy));
    }
}

fn handle_actions(
    mut actions: EventReader<Action>,
    mut positions: Query<&mut GridPosition>,
    mut healthy: Query<&mut Health>,
    mut world: ResMut<WorldMap>,
    mut kills: EventWriter<Kill>,
    names: Query<&Name>,
    mut log: EventWriter<LogMessage>,
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
            Action::Attack(attacker, attackee, damage) => {
                log.send(LogMessage(format!(
                    "{} attacks {}, dealing {} damage!",
                    names.get(*attacker).unwrap().capitalized(),
                    names.get(*attackee).unwrap().0,
                    damage
                )));

                let health = &mut healthy.get_mut(*attackee).unwrap().current;
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
    player: Query<(), With<Player>>,
    mut temp_app_exit_events: EventWriter<AppExit>,
    names: Query<&Name>,
    mut log: EventWriter<LogMessage>,
) {
    for entity in kills.iter().map(|k| k.0) {
        log.send(LogMessage(format!(
            "{} died!",
            names.get(entity).unwrap().capitalized()
        )));

        if player.get(entity).is_ok() {
            temp_app_exit_events.send(AppExit);
            return;
        }

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
