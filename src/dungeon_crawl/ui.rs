use super::{Cursor, Health, Name, Player};
use crate::{
    ui,
    world_map::{Grid, GridPosition, TileFlags, WorldMap},
};
use bevy::{
    math::vec2,
    prelude::*,
    render::camera::{Camera, OrthographicProjection},
};
use std::collections::VecDeque;

pub struct LogMessage(pub String);

pub fn update_health(
    mut text: Query<&mut Text, With<ui::HpText>>,
    mut bar: Query<&mut Style, With<ui::HpBar>>,
    hp: Query<&Health, With<Player>>,
) {
    let hp = match hp.single() {
        Ok(hp) => hp,
        Err(_) => return,
    };

    text.single_mut().unwrap().sections[0].value = format!("HP: {}/{}", hp.current, hp.max);
    bar.single_mut().unwrap().size.width = Val::Percent(100.0 * hp.current as f32 / hp.max as f32);
}

pub fn update_log(
    mut text: Query<&mut Text, With<ui::Log>>,
    mut messages: EventReader<LogMessage>,
    mut log: Local<VecDeque<String>>,
) {
    for m in messages.iter() {
        log.push_front(m.0.clone());
    }
    log.resize(6, String::from(" "));

    text.single_mut().unwrap().sections[0].value = log
        .iter()
        .intersperse(&String::from("\n"))
        .cloned()
        .collect();
}

pub fn update_cursor(
    windows: Res<Windows>,
    camera: Query<(&Transform, &OrthographicProjection), (With<Camera>, Without<ui::Camera>)>,
    grid: Res<Grid>,
    player: Query<&Player>,
    mut cursor: Query<(&mut GridPosition, &mut Visible), With<Cursor>>,
) {
    let window = windows.get_primary().unwrap();

    if let Some(pos) = window.cursor_position() {
        let size = Vec2::new(window.width(), window.height());
        let (camera, orto) = camera.single().unwrap();
        let pos = (pos - size / 2.0) * orto.scale;
        let world_pos = camera.compute_matrix() * pos.extend(0.0).extend(1.0);
        let grid_pos =
            (vec2(world_pos.x, world_pos.y) / grid.cell_size.as_f32() + vec2(0.5, 0.5)).as_i32();

        *cursor.single_mut().unwrap().0 = GridPosition {
            x: grid_pos.x,
            y: grid_pos.y,
        };
    }

    if let Ok(player) = player.single() {
        cursor.single_mut().unwrap().1.is_visible = player.selected.is_some();
    }
}

pub fn update_details(
    mut text: Query<&mut Text, With<ui::Details>>,
    names: Query<&Name>,
    health: Query<&Health>,
    world: Res<WorldMap>,
    cursor: Query<&GridPosition, With<Cursor>>,
) {
    let grid_pos = cursor.single().unwrap();

    if let Some(tile) = world.tiles.get(grid_pos.x, grid_pos.y) {
        if tile.contains(TileFlags::IN_VIEW) {
            if let Some(entities) = world.entities.get(grid_pos.x, grid_pos.y) {
                let mut details = vec![];
                for entity in entities {
                    let name = names.get(*entity).unwrap().capitalized();
                    let health = health
                        .get(*entity)
                        .map_or(String::from(""), |h| format!(" ({}/{})", h.current, h.max));
                    details.push(format!("{}{}", name, health));
                }

                details.resize(4, String::from(" "));
                text.single_mut().unwrap().sections[0].value = details
                    .into_iter()
                    .intersperse(String::from("\n"))
                    .collect();

                return;
            }
        }
    }
    // Else
    text.single_mut().unwrap().sections[0].value = String::from(" \n \n \n ");
}

pub fn update_inventory(
    mut text: Query<&mut Text, With<ui::Inventory>>,
    player: Query<&Player>,
    names: Query<&Name>,
) {
    if let Ok(player) = player.single() {
        let inventory = player.inventory;
        let ind = player.selected.unwrap_or(usize::MAX);

        let mut inv = vec![];
        for (i, e) in inventory.iter().enumerate() {
            inv.push(format!(
                "{} {}",
                if i == ind {
                    String::from(">>> ")
                } else {
                    format!("{}.", i + 1)
                },
                e.map_or(String::from(""), |e| names.get(e).unwrap().capitalized())
            ));
        }

        if inventory.iter().all(|i| i.is_none()) {
            text.single_mut().unwrap().sections[0].value =
                String::from("Press G to pick up items\n \n \n \n ");
        } else {
            text.single_mut().unwrap().sections[0].value =
                inv.into_iter().intersperse(String::from("\n")).collect();
        }
    }
}
