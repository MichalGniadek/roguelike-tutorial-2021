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
use rand::random;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnState {
    WorldUpdate,
    Turn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ev {
    Move(Entity, GridPosition, GridPosition),
    Attack(Entity, Entity, i32),
    PickUpItem(Entity, Entity),
    DropItem(Entity, Entity, GridPosition),
    Heal(Entity, i32),
    Paralyze(Entity, i32),
    RemoveFromMap(Entity),
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
        app.add_event::<Ev>()
            .add_plugin(ui::DungeonCrawlUIPlugin)
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
                .before("evs")
                .with_system(player_control.system())
                .with_system(enemy_ai.system())
                .with_system(paralyzed.system()),
        );

        app.add_system_set(
            SystemSet::on_update(AppState::DungeonCrawl(TurnState::Turn))
                .label("evs")
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

impl GameData {
    const MAP_SIZE: [(u32, (u32, u32)); 2] = [(1, (200, 400)), (4, (400, 600))];

    const ENEMY_COUNT: [(u32, u32); 3] = [(1, 3), (2, 4), (4, 6)];
    const ITEM_COUNT: [(u32, u32); 3] = [(1, 2), (2, 3), (4, 4)];

    const ITEM_CHANCES: [(u32, (Item, i32)); 4] = [
        (1, (Item::HealthPotion, 10)),
        (2, (Item::ScrollOfLightning, 5)),
        (4, (Item::ScrollOfFireball, 5)),
        (4, (Item::ScrollOfParalysis, 5)),
    ];

    pub fn floor_map_size(&self) -> (u32, u32) {
        self.calculate_count(Self::MAP_SIZE)
    }

    pub fn floor_enemy_count(&self) -> u32 {
        self.calculate_count(Self::ENEMY_COUNT)
    }

    pub fn floor_item_count(&self) -> u32 {
        self.calculate_count(Self::ITEM_COUNT)
    }

    pub fn floor_item(&self) -> Item {
        let mut map = HashMap::new();
        for (floor, (item, chance)) in Self::ITEM_CHANCES {
            if floor > self.floor {
                break;
            }
            map.insert(item, chance);
        }

        let sum: i32 = map.values().sum();
        let mut rand = 1 + random::<i32>() % sum;

        for (item, chance) in map {
            rand -= chance;
            if rand <= 0 {
                return item;
            }
        }
        unreachable!()
    }

    fn calculate_count<T: Copy, const N: usize>(&self, arr: [(u32, T); N]) -> T {
        arr.iter()
            .find(|(floor, _)| *floor < self.floor + 1)
            .unwrap()
            .1
    }
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
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Item {
    HealthPotion,
    ScrollOfLightning,
    ScrollOfParalysis,
    ScrollOfFireball,
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
                    evs.send(Ev::PickUpItem(player_entity, item));
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
                        Item::HealthPotion => {
                            if let Some(e) = world.entities[cursor]
                                .iter()
                                .find(|e| healthy_entities.get(**e).is_ok())
                            {
                                evs.send(Ev::Heal(*e, 4));
                                inventory.inventory[index] = None;
                                inventory.selected = None;
                            }
                        }
                        Item::ScrollOfLightning => {
                            if let Some(e) = world.entities[cursor]
                                .iter()
                                .find(|e| healthy_entities.get(**e).is_ok())
                            {
                                evs.send(Ev::Attack(player_entity, *e, 2));
                                inventory.inventory[index] = None;
                                inventory.selected = None;
                            }
                        }
                        Item::ScrollOfParalysis => {
                            if let Some(e) = world.entities[cursor]
                                .iter()
                                .find(|e| controllers.get(**e).is_ok())
                            {
                                evs.send(Ev::Paralyze(*e, 4));
                                inventory.inventory[index] = None;
                                inventory.selected = None;
                            }
                        }
                        Item::ScrollOfFireball => {
                            for x in -1..=1 {
                                for y in -1..=1 {
                                    if let Some(e) = world.entities[[cursor.x + x, cursor.y + y]]
                                        .iter()
                                        .find(|e| healthy_entities.get(**e).is_ok())
                                    {
                                        evs.send(Ev::Attack(player_entity, *e, 1));
                                        inventory.inventory[index] = None;
                                        inventory.selected = None;
                                    }
                                }
                            }
                        }
                    }
                } else if buttons.just_pressed(MouseButton::Right) {
                    evs.send(Ev::DropItem(player_entity, item, cursor));
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
                    evs.send(Ev::Attack(player_entity, entity, 1));
                }
            }
        } else {
            evs.send(Ev::Move(player_entity, *position, new_pos));
        }
    }
}

fn enemy_ai(
    enemy: Query<(Entity, &GridPosition), (With<EnemyAI>, With<Initiative>, Without<Paralyzed>)>,
    player: Query<(Entity, &GridPosition), With<Player>>,
    world: Res<WorldMap>,
    mut evs: EventWriter<Ev>,
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
                evs.send(Ev::Attack(enemy, player, 1));
            } else if !world.tiles[path[1]].contains(TileFlags::BLOCKS_MOVEMENT) {
                evs.send(Ev::Move(enemy, *position, path[1]));
            } else {
                evs.send(Ev::Nothing);
            }
        } else {
            evs.send(Ev::Nothing);
        }
    } else {
        evs.send(Ev::Nothing);
    }
}

fn paralyzed(
    mut paralyzed: Query<(Entity, &mut Paralyzed), With<Initiative>>,
    mut evs: EventWriter<Ev>,
    mut commands: Commands,
) {
    if let Ok((entity, mut paralyzed)) = paralyzed.single_mut() {
        paralyzed.0 -= 1;
        if paralyzed.0 == 0 {
            commands.entity(entity).remove::<Paralyzed>();
        } else {
            evs.send(Ev::Nothing);
        }
    }
}

fn handle_evs(
    mut events: EventReader<Ev>,
    mut positions: Query<&mut GridPosition>,
    mut healthy: Query<&mut Health>,
    mut world: ResMut<WorldMap>,
    names: Query<&Name>,
    player: Query<(), With<Player>>,
    mut log: EventWriter<LogMessage>,
    mut app_state: ResMut<State<AppState>>,
    mut data: ResMut<GameData>,
    mut visible: Query<&mut Visible>,
    mut commands: Commands,
    mut order: ResMut<InitiativeOrder>,
) {
    let mut evs: VecDeque<Ev> = VecDeque::new();
    evs.extend(events.iter());

    let mut next_app_state = None;
    if !evs.is_empty() {
        next_app_state = Some(AppState::DungeonCrawl(TurnState::WorldUpdate));
    }

    while let Some(ev) = evs.pop_front() {
        match ev {
            Ev::Nothing => {}
            Ev::Move(entity, old_pos, new_pos) => {
                let i = world.entities[old_pos]
                    .iter()
                    .position(|x| *x == entity)
                    .unwrap();
                world.entities[old_pos].swap_remove(i);
                world.entities[new_pos].push(entity);

                if let Ok(mut pos) = positions.get_mut(entity) {
                    *pos = new_pos;
                }
            }
            Ev::Attack(attacker, attackee, damage) => {
                log.send(LogMessage(format!(
                    "{} attacks {}, dealing {} damage!",
                    names.get(attacker).unwrap().capitalized(),
                    names.get(attackee).unwrap().0,
                    damage
                )));

                let health = &mut healthy.get_mut(attackee).unwrap().current;
                *health -= damage;

                if *health <= 0 {
                    log.send(LogMessage(format!(
                        "{} died!",
                        names.get(attackee).unwrap().capitalized()
                    )));

                    if player.get(attacker).is_ok() {
                        data.current_xp += 1;
                        if data.current_xp >= data.needed_xp {
                            log.send(LogMessage("You level up!".into()));

                            data.current_xp = 0;
                            data.needed_xp += 2;
                            data.level += 1;

                            healthy.get_mut(attacker).unwrap().max += 2;
                        }
                    }

                    evs.push_back(Ev::RemoveFromMap(attackee));
                    evs.push_back(Ev::RemoveFromInitiative(attackee));
                    evs.push_back(Ev::Despawn(attackee));
                }
            }
            Ev::PickUpItem(_, item) => {
                for slot in &mut data.inventory {
                    if slot.is_none() {
                        *slot = Some(item);
                        log.send(LogMessage(format!(
                            "You pick up {}.",
                            names.get(item).unwrap().0,
                        )));
                        evs.push_back(Ev::RemoveFromMap(item));
                        break;
                    }
                }
            }
            Ev::DropItem(_, item, position) => {
                if world.tiles[position].contains(TileFlags::BLOCKS_MOVEMENT) {
                    log.send(LogMessage(format!(
                        "{} slams into the wall.",
                        names.get(item).unwrap().capitalized(),
                    )));
                    evs.push_back(Ev::Despawn(item));
                } else {
                    log.send(LogMessage(format!(
                        "{} lands on the floor.",
                        names.get(item).unwrap().capitalized(),
                    )));
                    evs.push_back(Ev::AddToMap(item, position));
                }
            }
            Ev::Heal(entity, amount) => {
                log.send(LogMessage(format!(
                    "{} is healed by {} health.",
                    names.get(entity).unwrap().capitalized(),
                    amount
                )));
                let mut hp = healthy.get_mut(entity).unwrap();
                hp.current = i32::min(hp.max, hp.current + amount);
            }
            Ev::RemoveFromMap(entity) => {
                let pos = positions.get_mut(entity).unwrap();
                let i = world.entities[*pos]
                    .iter()
                    .position(|x| *x == entity)
                    .unwrap();
                world.entities[*pos].swap_remove(i);
                commands.entity(entity).remove::<GridPosition>();
                visible.get_mut(entity).unwrap().is_visible = false;
            }
            Ev::AddToMap(entity, position) => {
                world.entities[position].push(entity);
                commands.entity(entity).insert(position);
                visible.get_mut(entity).unwrap().is_visible = true;
            }
            Ev::RemoveFromInitiative(entity) => {
                let i = order.0.iter().position(|x| *x == entity).unwrap();
                order.0.remove(i);
                commands.entity(entity).remove::<Initiative>();
            }
            Ev::Despawn(entity) => {
                if player.get(entity).is_ok() {
                    next_app_state = Some(AppState::DungeonCrawlExitToMenu);
                } else {
                    commands.entity(entity).despawn();
                }
            }
            Ev::Paralyze(entity, duration) => {
                commands.entity(entity).insert(Paralyzed(duration));
            }
            Ev::Quit => {
                next_app_state = Some(AppState::DungeonCrawlExitToMenu);
            }
            Ev::Descend => {
                log.send(LogMessage("You descend to the next dungeon floor".into()));
                next_app_state = Some(AppState::DungeonCrawlDescend);
            }
        }
    }

    if let Some(state) = next_app_state {
        app_state.set(state).unwrap();
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
