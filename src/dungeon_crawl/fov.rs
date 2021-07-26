use super::Player;
use crate::world_map::{GridPosition, Tile, TileFlags, WorldMap};
use bevy::prelude::*;

pub fn player_fov(
    player: Query<&GridPosition, With<Player>>,
    mut visible: Query<(&mut Visible, &GridPosition, Option<&Tile>)>,
    mut tiles: Query<(&mut Handle<ColorMaterial>, &GridPosition), With<Tile>>,
    mut world: ResMut<WorldMap>,
) {
    let position = match player.single() {
        Ok(position) => position.clone(),
        Err(_) => return,
    };

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
