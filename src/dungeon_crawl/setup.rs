use super::{EnemyAI, Initiative, Player};
use crate::world_map::{
    BlocksMovement, BlocksVision, Grid, GridPosition, Tile, TileFlags, WorldMap,
};
use bevy::{prelude::*, render::camera::Camera};
use std::collections::VecDeque;

pub fn update_position(
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

pub fn camera_position(
    mut query: QuerySet<(
        Query<&Transform, With<Player>>,
        Query<&mut Transform, With<Camera>>,
    )>,
) {
    let mut position = match query.q0_mut().single_mut() {
        Ok(position) => position.clone(),
        Err(_) => return,
    };
    let mut camera = query.q1_mut().single_mut().unwrap();
    position.translation.z = camera.translation.z;
    *camera = position;
}

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
