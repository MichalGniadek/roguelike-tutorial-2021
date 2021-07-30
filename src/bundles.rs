use crate::{
    dungeon_crawl::{EnemyAI, Health, Item, Name, Player},
    world_map::BlocksMovement,
};
use bevy::prelude::*;

#[derive(Bundle)]
pub struct EnemyBundle {
    #[bundle]
    sprite: SpriteBundle,
    _e: EnemyAI,
    _bm: BlocksMovement,
    health: Health,
    name: Name,
}

impl EnemyBundle {
    pub fn orc(asset_server: &AssetServer, materials: &mut ResMut<Assets<ColorMaterial>>) -> Self {
        Self {
            sprite: SpriteBundle {
                material: materials.add(ColorMaterial {
                    texture: Some(asset_server.load("orc-head.png")),
                    color: Color::hex("DA0037").unwrap(),
                }),
                transform: Transform::from_xyz(0.0, 0.0, 1.0),
                ..Default::default()
            },
            _e: EnemyAI,
            _bm: BlocksMovement,
            health: Health::new(3, 3),
            name: Name(String::from("orc")),
        }
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    #[bundle]
    sprite: SpriteBundle,
    player: Player,
    health: Health,
    name: Name,
}

impl PlayerBundle {
    pub fn new(asset_server: &AssetServer, materials: &mut ResMut<Assets<ColorMaterial>>) -> Self {
        Self {
            sprite: SpriteBundle {
                material: materials.add(ColorMaterial {
                    texture: Some(asset_server.load("hooded-figure.png")),
                    color: Color::hex("EDEDED").unwrap(),
                }),
                transform: Transform::from_xyz(0.0, 0.0, 1.0),
                ..Default::default()
            },
            player: Player {
                inventory: [None; 5],
            },
            health: Health::new(8, 8),
            name: Name(String::from("player")),
        }
    }
}

#[derive(Bundle)]
pub struct ItemBundle {
    #[bundle]
    sprite: SpriteBundle,
    item: Item,
    name: Name,
}

impl ItemBundle {
    pub fn health_potion(
        asset_server: &AssetServer,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> Self {
        Self {
            sprite: SpriteBundle {
                material: materials.add(ColorMaterial {
                    texture: Some(asset_server.load("potion-ball.png")),
                    color: Color::hex("DA0037").unwrap(),
                }),
                transform: Transform::from_xyz(0.0, 0.0, 1.0),
                ..Default::default()
            },
            item: Item::HealthPotion(4),
            name: Name(String::from("health potion")),
        }
    }
}
