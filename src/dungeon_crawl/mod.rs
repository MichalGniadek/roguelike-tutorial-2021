mod fov;
mod setup;
mod ui;

use self::ui::{Logs, MyCanvas};
use crate::{
    dungeon_crawl::ui::LogMessage,
    world_map::{GridPosition, TileFlags, WorldMap},
    AppState,
};
use bevy::{ecs::system::QuerySingleError, prelude::*};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnState {
    WorldUpdate,
    Turn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Wait,
    Move(Entity, GridPosition, GridPosition),
    Attack(Entity, Entity, i32),
    PickUpItem(Entity, Entity),
    DropItem(Entity, Entity, GridPosition),
    Heal(Entity, i32),
    Paralyze(Entity, i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ev {
    RemoveFromMap(Entity),
    Paralyze(Entity, i32),
    AddToMap(Entity, GridPosition),
    RemoveFromInitiative(Entity),
    Despawn(Entity),
    Nothing,
    Quit,
    Descend,
}

#[derive(Default, Clone)]
pub struct InitiativeOrder(pub VecDeque<Entity>);

pub struct DungeonCrawlPlugin;
impl Plugin for DungeonCrawlPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<Action>()
            .add_event::<Ev>()
            .add_plugin(ui::DungeonCrawlUIPlugin)
            .init_resource::<Logs>()
            .init_resource::<GameData>()
            .init_resource::<InitiativeOrder>();

        macro_rules! switch_app_state {
            ($e:expr) => {
                (|mut app_state: ResMut<State<AppState>>| {
                    let _ = app_state.set($e);
                })
            };
        }

        app.add_system_set(
            SystemSet::on_enter(AppState::DungeonCrawlEnter).with_system(
                switch_app_state!(AppState::DungeonCrawl(TurnState::WorldUpdate)).system(),
            ),
        );
        app.add_system_set(
            SystemSet::on_enter(AppState::DungeonCrawlExitToMenu)
                .with_system(cleanup.system())
                .with_system(cleanup_log_and_inventory.system())
                .with_system(switch_app_state!(AppState::MainMenu).system()),
        );
        app.add_system_set(
            SystemSet::on_enter(AppState::DungeonCrawlDescend)
                .with_system(cleanup.system())
                .with_system(switch_app_state!(AppState::WorldGeneration).system()),
        );

        use fov::*;
        use setup::*;
        app.add_system_set(
            SystemSet::on_enter(AppState::DungeonCrawl(TurnState::WorldUpdate))
                .with_system(update_world_map.system().label("update_world_map"))
                .with_system(handle_initiative.system())
                .with_system(player_fov.system().after("update_world_map"))
                .with_system(switch_app_state!(AppState::DungeonCrawl(TurnState::Turn)).system()),
        );

        app.add_system_set(
            SystemSet::on_update(AppState::DungeonCrawl(TurnState::Turn))
                .before("actions")
                .with_system(player_control.system())
                .with_system(enemy_ai.system())
                .with_system(paralyzed.system()),
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
    }
}

pub struct GameData {
    pub inventory: [Option<Entity>; 5],
    pub selected: Option<usize>,

    pub previous_hp: Option<Health>,
    pub floor: u32,

    pub level: u32,
    pub current_xp: u32,
    pub needed_xp: u32,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            inventory: [None; 5],
            selected: None,

            previous_hp: None,
            floor: 1,

            level: 1,
            current_xp: 0,
            needed_xp: 3,
        }
    }
}

pub struct Player;
pub struct EnemyAI;
pub struct Initiative;
pub struct Name(pub String);
pub enum Item {
    HealthPotion(i32),
    ScrollOfLightning(i32),
    ScrollOfParalysis(i32),
    ScrollOfFireball(i32),
}
pub struct Paralyzed(i32);
pub struct Cursor;

impl Name {
    pub fn capitalized(&self) -> String {
        let mut chars = self.0.chars();
        let first = chars.next().unwrap().to_uppercase();
        format!("{}{}", first.collect::<String>(), chars.collect::<String>())
    }
}

#[derive(Clone, Copy)]
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
    mut query: Query<(Entity, &GridPosition), (With<Initiative>, Without<Paralyzed>, With<Player>)>,
    healthy_entities: Query<(), With<Health>>,
    mut inventory: ResMut<GameData>,
    world: Res<WorldMap>,
    keys: Res<Input<KeyCode>>,
    buttons: Res<Input<MouseButton>>,
    items: Query<(Entity, Option<&GridPosition>, &Item)>,
    mut actions: EventWriter<Action>,
    cursor: Query<&GridPosition, With<Cursor>>,
    controllers: Query<Entity, Or<(With<Player>, With<EnemyAI>)>>,
    mut evs: EventWriter<Ev>,
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
            Some(KeyCode::G) => {
                inventory.selected = None;
                if let Some((item, _, _)) =
                    items.iter().find(|(_, item, _)| item.contains(&position))
                {
                    actions.send(Action::PickUpItem(player_entity, item));
                }
                return;
            }
            Some(KeyCode::Key1) => inventory.selected = Some(0),
            Some(KeyCode::Key2) => inventory.selected = Some(1),
            Some(KeyCode::Key3) => inventory.selected = Some(2),
            Some(KeyCode::Key4) => inventory.selected = Some(3),
            Some(KeyCode::Key5) => inventory.selected = Some(4),
            Some(KeyCode::Escape) => evs.send(Ev::Quit),
            _ => {}
        }
    }

    let cursor = *cursor.single().unwrap();
    if world.tiles[cursor].contains(TileFlags::IN_VIEW) {
        if let Some(index) = inventory.selected {
            if let Some(item) = inventory.inventory[index] {
                if buttons.just_pressed(MouseButton::Left) {
                    match items.get(item).unwrap().2 {
                        Item::HealthPotion(amount) => {
                            if let Some(e) = world.entities[cursor]
                                .iter()
                                .find(|e| healthy_entities.get(**e).is_ok())
                            {
                                actions.send(Action::Heal(*e, *amount));
                                inventory.inventory[index] = None;
                                inventory.selected = None;
                            }
                        }
                        Item::ScrollOfLightning(damage) => {
                            if let Some(e) = world.entities[cursor]
                                .iter()
                                .find(|e| healthy_entities.get(**e).is_ok())
                            {
                                actions.send(Action::Attack(player_entity, *e, *damage));
                                inventory.inventory[index] = None;
                                inventory.selected = None;
                            }
                        }
                        Item::ScrollOfParalysis(duration) => {
                            if let Some(e) = world.entities[cursor]
                                .iter()
                                .find(|e| controllers.get(**e).is_ok())
                            {
                                actions.send(Action::Paralyze(*e, *duration));
                                inventory.inventory[index] = None;
                                inventory.selected = None;
                            }
                        }
                        Item::ScrollOfFireball(damage) => {
                            for x in -1..=1 {
                                for y in -1..=1 {
                                    if let Some(e) = world.entities[[cursor.x + x, cursor.y + y]]
                                        .iter()
                                        .find(|e| healthy_entities.get(**e).is_ok())
                                    {
                                        actions.send(Action::Attack(player_entity, *e, *damage));
                                        inventory.inventory[index] = None;
                                        inventory.selected = None;
                                    }
                                }
                            }
                        }
                    }
                } else if buttons.just_pressed(MouseButton::Right) {
                    actions.send(Action::DropItem(player_entity, item, cursor));
                    inventory.inventory[index] = None;
                    inventory.selected = None;
                }
            }
        }
    }

    if let Some(i) = inventory.selected {
        if inventory.inventory[i].is_none() {
            inventory.selected = None;
        }
    }

    if *position != new_pos {
        inventory.selected = None;

        if new_pos == world.stairs {
            evs.send(Ev::Descend);
        } else if world.tiles[new_pos].contains(TileFlags::BLOCKS_MOVEMENT) {
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
    enemy: Query<(Entity, &GridPosition), (With<EnemyAI>, With<Initiative>, Without<Paralyzed>)>,
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
                actions.send(Action::Wait);
            }
        } else {
            actions.send(Action::Wait);
        }
    } else {
        actions.send(Action::Wait);
    }
}

fn paralyzed(
    mut paralyzed: Query<(Entity, &mut Paralyzed), With<Initiative>>,
    mut actions: EventWriter<Action>,
    mut commands: Commands,
) {
    if let Ok((entity, mut paralyzed)) = paralyzed.single_mut() {
        paralyzed.0 -= 1;
        if paralyzed.0 == 0 {
            commands.entity(entity).remove::<Paralyzed>();
        } else {
            actions.send(Action::Wait);
        }
    }
}

fn handle_actions(
    mut actions: EventReader<Action>,
    mut positions: Query<&mut GridPosition>,
    mut healthy: Query<&mut Health>,
    mut world: ResMut<WorldMap>,
    mut evs: EventWriter<Ev>,
    names: Query<&Name>,
    player: Query<(), With<Player>>,
    mut log: EventWriter<LogMessage>,
    mut data: ResMut<GameData>,
) {
    for a in actions.iter() {
        match a {
            Action::Wait => evs.send(Ev::Nothing),
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

                if *health <= 0 {
                    log.send(LogMessage(format!(
                        "{} died!",
                        names.get(*attackee).unwrap().capitalized()
                    )));

                    if player.get(*attacker).is_ok() {
                        data.current_xp += 1;
                        if data.current_xp >= data.needed_xp {
                            log.send(LogMessage("You level up!".into()));

                            data.current_xp = 0;
                            data.needed_xp += 2;
                            data.level += 1;

                            healthy.get_mut(*attacker).unwrap().max += 2;
                        }
                    }

                    evs.send(Ev::RemoveFromMap(*attackee));
                    evs.send(Ev::RemoveFromInitiative(*attackee));
                    evs.send(Ev::Despawn(*attackee));
                } else {
                    evs.send(Ev::Nothing);
                }
            }
            Action::PickUpItem(_, item) => {
                for slot in &mut data.inventory {
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
            Action::DropItem(_, item, position) => {
                if world.tiles[*position].contains(TileFlags::BLOCKS_MOVEMENT) {
                    log.send(LogMessage(format!(
                        "{} slams into the wall.",
                        names.get(*item).unwrap().capitalized(),
                    )));
                    evs.send(Ev::Despawn(*item));
                } else {
                    log.send(LogMessage(format!(
                        "{} lands on the floor.",
                        names.get(*item).unwrap().capitalized(),
                    )));
                    evs.send(Ev::AddToMap(*item, *position));
                }
            }
            Action::Heal(entity, amount) => {
                log.send(LogMessage(format!(
                    "{} is healed by {} health.",
                    names.get(*entity).unwrap().capitalized(),
                    amount
                )));
                let mut hp = healthy.get_mut(*entity).unwrap();
                hp.current = i32::min(hp.max, hp.current + amount);
                evs.send(Ev::Nothing);
            }
            Action::Paralyze(entity, duration) => evs.send(Ev::Paralyze(*entity, *duration)),
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
    mut app_state: ResMut<State<AppState>>,
    mut log: EventWriter<LogMessage>,
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
                    app_state.set(AppState::DungeonCrawlExitToMenu).unwrap();
                    return;
                }
                commands.entity(*entity).despawn();
            }
            Ev::Nothing => {}
            Ev::Paralyze(entity, duration) => {
                commands.entity(*entity).insert(Paralyzed(*duration));
            }
            Ev::Quit => {
                app_state.set(AppState::DungeonCrawlExitToMenu).unwrap();
                return;
            }
            Ev::Descend => {
                log.send(LogMessage("You descend to the next dungeon floor".into()));
                app_state.set(AppState::DungeonCrawlDescend).unwrap();
                return;
            }
        }
    }

    if any_evs {
        app_state
            .set(AppState::DungeonCrawl(TurnState::WorldUpdate))
            .unwrap();
    }
}

pub fn cleanup(
    // Player might have died so it has additional check
    q: Query<Entity, Or<(With<MyCanvas>, With<GridPosition>, With<Player>)>>,
    mut commands: Commands,
    mut data: ResMut<GameData>,
    player: Query<&Health, With<Player>>,
) {
    for e in q.iter() {
        commands.entity(e).despawn_recursive();
    }
    commands.remove_resource::<InitiativeOrder>();
    commands.remove_resource::<WorldMap>();
    data.previous_hp = Some(*player.single().unwrap());
    data.floor += 1;
}

pub fn cleanup_log_and_inventory(mut commands: Commands, inventory: Res<GameData>) {
    commands.insert_resource(Logs::default());
    for e in inventory.inventory.iter().filter_map(|i| *i) {
        commands.entity(e).despawn();
    }
    commands.insert_resource(GameData::default());
}
