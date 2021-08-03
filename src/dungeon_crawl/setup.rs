use super::{EnemyAI, Initiative, Player};
use crate::world_map::{BlocksMovement, BlocksVision, Tile, TileFlags, WorldMap};
use bevy::prelude::*;
use std::collections::VecDeque;

pub fn update_world_map(
    mut world: ResMut<WorldMap>,
    t: Query<(&Tile, &BlocksMovement)>,
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
            if world.entities[[x, y]]
                .iter()
                .any(|e| matches!(t.get(*e), Ok((&Tile, &BlocksMovement))))
            {
                world.tiles[[x, y]] |= TileFlags::BLOCKS_PATHFINDING;
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

#[derive(Default)]
pub struct InitiativeOrder(pub VecDeque<Entity>);

pub fn handle_initiative(
    mut order: ResMut<InitiativeOrder>,
    curr: Query<Entity, With<Initiative>>,
    characters: Query<Entity, Or<(With<Player>, With<EnemyAI>)>>,
    mut commands: Commands,
) {
    if let Ok(entity) = curr.single() {
        commands.entity(entity).remove::<Initiative>();
    }

    for c in characters.iter() {
        if !order.0.contains(&c) {
            order.0.push_back(c);
        }
    }

    if let Some(entity) = order.0.pop_front() {
        commands.entity(entity).insert(Initiative);
        order.0.push_back(entity);
    }
}
