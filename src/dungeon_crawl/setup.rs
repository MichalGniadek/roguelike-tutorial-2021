use super::{Enemy, Initiative, InitiativeOrder, Player};
use crate::world_map::{BlocksMovement, BlocksVision, Grid, GridPosition, TileFlags, WorldMap};
use bevy::{prelude::*, render::camera::Camera};

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
    let mut position = query.q0_mut().single_mut().unwrap().clone();
    let mut camera = query.q1_mut().single_mut().unwrap();
    position.translation.z = camera.translation.z;
    *camera = position;
}

pub fn update_world_map(
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

pub fn handle_initiative(
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
