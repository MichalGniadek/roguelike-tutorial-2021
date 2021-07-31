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
    PickUpItem(Entity, Entity),
    UseItem(Entity, Entity, GridPosition),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ev {
    RemoveFromMap(Entity),
    AddToMap(Entity, GridPosition),
    RemoveFromInitiative(Entity),
    Despawn(Entity),
    Nothing,
}

pub struct DungeonCrawlPlugin;
impl Plugin for DungeonCrawlPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<Action>()
            .add_event::<Ev>()
            .add_event::<LogMessage>()
            .init_resource::<InitiativeOrder>();

        use fov::*;
        use setup::*;
        app.add_system_set(
            SystemSet::on_enter(AppState::DungeonCrawl(TurnState::Setup))
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
                .with_system(handle_evs.system()),
        );
        app.add_system_set(
            SystemSet::on_update(AppState::DungeonCrawl(TurnState::Turn))
                .with_system(update_position.system().label("positions"))
                .with_system(camera_position.system().after("positions"))
                .with_system(ui::update_health.system())
                .with_system(ui::update_log.system())
                .with_system(ui::update_cursor.system().before("positions"))
                .with_system(ui::update_details.system())
                .with_system(ui::update_inventory.system()),
        );
    }
}

pub struct Player {
    pub inventory: [Option<Entity>; 5],
    pub selected: Option<usize>,
}
pub struct EnemyAI;
pub struct Initiative;
pub struct Name(pub String);
pub enum Item {
    HealthPotion(i32),
}
pub struct Cursor;

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
    mut query: Query<(Entity, &mut Player, &GridPosition), With<Initiative>>,
    healthy_entities: Query<(), With<Health>>,
    world: Res<WorldMap>,
    keys: Res<Input<KeyCode>>,
    buttons: Res<Input<MouseButton>>,
    items: Query<(Entity, &GridPosition), With<Item>>,
    mut actions: EventWriter<Action>,
    cursor: Query<&GridPosition, With<Cursor>>,
) {
    let (player_entity, mut player, position) = match query.single_mut() {
        Ok((e, p, pos)) => (e, p, pos),
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
            Some(KeyCode::G) => {
                player.selected = None;
                if let Some((item, _)) = items.iter().find(|(_, item)| *item == position) {
                    actions.send(Action::PickUpItem(player_entity, item));
                }
                return;
            }
            Some(KeyCode::Key1) => player.selected = Some(0),
            Some(KeyCode::Key2) => player.selected = Some(1),
            Some(KeyCode::Key3) => player.selected = Some(2),
            Some(KeyCode::Key4) => player.selected = Some(3),
            Some(KeyCode::Key5) => player.selected = Some(4),
            _ => {}
        }
    }

    if buttons.just_pressed(MouseButton::Left) {
        let cursor = *cursor.single().unwrap();
        if world.tiles[cursor].contains(TileFlags::IN_VIEW) {
            if let Some(index) = player.selected {
                if let Some(item) = player.inventory[index] {
                    actions.send(Action::UseItem(player_entity, item, cursor));
                    player.inventory[index] = None;
                    player.selected = None;
                    return;
                }
            }
        }
    }

    if let Some(i) = player.selected {
        if player.inventory[i].is_none() {
            player.selected = None;
        }
    }

    if *position != new_pos {
        player.selected = None;
        if world.tiles[new_pos].contains(TileFlags::BLOCKS_MOVEMENT) {
            for &entity in &world.entities[new_pos] {
                if let Ok(()) = healthy_entities.get(entity) {
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
    mut evs: EventWriter<Ev>,
    names: Query<&Name>,
    mut log: EventWriter<LogMessage>,
    mut player: Query<&mut Player>,
    items: Query<&Item>,
) {
    for a in actions.iter() {
        match a {
            Action::Wait(_) => evs.send(Ev::Nothing),
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
                evs.send(Ev::Nothing);
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
                    log.send(LogMessage(format!(
                        "{} died!",
                        names.get(*attackee).unwrap().capitalized()
                    )));

                    evs.send(Ev::RemoveFromMap(*attackee));
                    evs.send(Ev::RemoveFromInitiative(*attackee));
                    evs.send(Ev::Despawn(*attackee));
                } else {
                    evs.send(Ev::Nothing);
                }
            }
            Action::PickUpItem(_, item) => {
                let inventory = &mut player.single_mut().unwrap().inventory;

                for slot in inventory {
                    if slot.is_none() {
                        *slot = Some(*item);
                        log.send(LogMessage(format!(
                            "You pick up {}.",
                            names.get(*item).unwrap().0,
                        )));
                        evs.send(Ev::RemoveFromMap(*item));
                        break;
                    }
                }
            }
            Action::UseItem(_user, item, position) => match items.get(*item).unwrap() {
                Item::HealthPotion(amount) => {
                    if let Some(e) = world.entities[*position]
                        .iter()
                        .find(|e| healthy.get_mut(**e).is_ok())
                    {
                        log.send(LogMessage(format!(
                            "{} is healed by {} health.",
                            names.get(*e).unwrap().capitalized(),
                            amount
                        )));
                        let mut e = healthy.get_mut(*e).unwrap();
                        e.current = i32::min(e.max, e.current + amount);
                        evs.send(Ev::Nothing);
                    } else if !world.tiles[*position].contains(TileFlags::BLOCKS_MOVEMENT) {
                        log.send(LogMessage(String::from(
                            "Health potion lands on the floor.",
                        )));
                        evs.send(Ev::AddToMap(*item, *position));
                    } else {
                        log.send(LogMessage(String::from(
                            "Health potion breaks on the wall.",
                        )));
                        evs.send(Ev::Despawn(*item));
                    }
                }
            },
        }
    }
}

fn handle_evs(
    mut evs: EventReader<Ev>,
    mut commands: Commands,
    mut order: ResMut<InitiativeOrder>,
    mut positions: Query<&mut GridPosition>,
    mut world: ResMut<WorldMap>,
    player: Query<(), With<Player>>,
    mut visible: Query<&mut Visible>,
    mut temp_app_exit_events: EventWriter<AppExit>,
    mut app_state: ResMut<State<AppState>>,
) {
    let mut any_evs = false;
    for ev in evs.iter() {
        any_evs = true;
        match ev {
            Ev::RemoveFromMap(entity) => {
                let pos = positions.get_mut(*entity).unwrap();
                let i = world.entities[*pos]
                    .iter()
                    .position(|x| x == entity)
                    .unwrap();
                world.entities[*pos].swap_remove(i);
                commands.entity(*entity).remove::<GridPosition>();
                visible.get_mut(*entity).unwrap().is_visible = false;
            }
            Ev::AddToMap(entity, position) => {
                world.entities[*position].push(*entity);
                commands.entity(*entity).insert(*position);
                visible.get_mut(*entity).unwrap().is_visible = true;
            }
            Ev::RemoveFromInitiative(entity) => {
                let i = order.0.iter().position(|x| x == entity).unwrap();
                order.0.remove(i);
                commands.entity(*entity).remove::<Initiative>();
            }
            Ev::Despawn(entity) => {
                if player.get(*entity).is_ok() {
                    temp_app_exit_events.send(AppExit);
                    return;
                }
                commands.entity(*entity).despawn();
            }
            Ev::Nothing => {}
        }
    }

    if any_evs {
        app_state
            .set(AppState::DungeonCrawl(TurnState::Setup))
            .unwrap();
    }
}
