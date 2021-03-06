use crate::{
    dungeon_crawl::{EnemyAI, GameData, Health, Item, Name, Player},
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
    pub fn new(
        asset_server: &AssetServer,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        data: &GameData,
    ) -> Self {
        Self {
            sprite: SpriteBundle {
                material: materials.add(ColorMaterial {
                    texture: Some(asset_server.load("hooded-figure.png")),
                    color: Color::hex("EDEDED").unwrap(),
                }),
                transform: Transform::from_xyz(0.0, 0.0, 1.0),
                ..Default::default()
            },
            player: Player,
            health: data.previous_hp.unwrap_or(Health::new(8, 8)),
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
    pub fn item(
        item: Item,
        asset_server: &AssetServer,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> Self {
        match item {
            Item::HealthPotion => Self::health_potion(asset_server, materials),
            Item::ScrollOfLightning => Self::scroll_of_lightning(asset_server, materials),
            Item::ScrollOfParalysis => Self::scroll_of_paralysis(asset_server, materials),
            Item::ScrollOfFireball => Self::scroll_of_fireball(asset_server, materials),
            Item::Sword => Self::sword(asset_server, materials),
            Item::WarAxe => Self::war_axe(asset_server, materials),
            Item::Armor => Self::armor(asset_server, materials),
        }
    }

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
            item: Item::HealthPotion,
            name: Name(String::from("health potion")),
        }
    }

    pub fn scroll_of_lightning(
        asset_server: &AssetServer,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> Self {
        Self {
            sprite: SpriteBundle {
                material: materials.add(ColorMaterial {
                    texture: Some(asset_server.load("scroll-unfurled.png")),
                    color: Color::hex("EDEDED").unwrap(),
                }),
                transform: Transform::from_xyz(0.0, 0.0, 1.0),
                ..Default::default()
            },
            item: Item::ScrollOfLightning,
            name: Name(String::from("scroll of lightning")),
        }
    }

    pub fn scroll_of_paralysis(
        asset_server: &AssetServer,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> Self {
        Self {
            sprite: SpriteBundle {
                material: materials.add(ColorMaterial {
                    texture: Some(asset_server.load("scroll-unfurled.png")),
                    color: Color::hex("EDEDED").unwrap(),
                }),
                transform: Transform::from_xyz(0.0, 0.0, 1.0),
                ..Default::default()
            },
            item: Item::ScrollOfParalysis,
            name: Name(String::from("scroll of paralysis")),
        }
    }

    pub fn scroll_of_fireball(
        asset_server: &AssetServer,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> Self {
        Self {
            sprite: SpriteBundle {
                material: materials.add(ColorMaterial {
                    texture: Some(asset_server.load("scroll-unfurled.png")),
                    color: Color::hex("EDEDED").unwrap(),
                }),
                transform: Transform::from_xyz(0.0, 0.0, 1.0),
                ..Default::default()
            },
            item: Item::ScrollOfFireball,
            name: Name(String::from("scroll of fireball")),
        }
    }

    pub fn war_axe(
        asset_server: &AssetServer,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> Self {
        Self {
            sprite: SpriteBundle {
                material: materials.add(ColorMaterial {
                    texture: Some(asset_server.load("battle-axe.png")),
                    color: Color::hex("EDEDED").unwrap(),
                }),
                transform: Transform::from_xyz(0.0, 0.0, 1.0),
                ..Default::default()
            },
            item: Item::WarAxe,
            name: Name(String::from("war axe")),
        }
    }

    pub fn sword(
        asset_server: &AssetServer,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> Self {
        Self {
            sprite: SpriteBundle {
                material: materials.add(ColorMaterial {
                    texture: Some(asset_server.load("gladius.png")),
                    color: Color::hex("EDEDED").unwrap(),
                }),
                transform: Transform::from_xyz(0.0, 0.0, 1.0),
                ..Default::default()
            },
            item: Item::Sword,
            name: Name(String::from("sword")),
        }
    }

    pub fn armor(
        asset_server: &AssetServer,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> Self {
        Self {
            sprite: SpriteBundle {
                material: materials.add(ColorMaterial {
                    texture: Some(asset_server.load("breastplate.png")),
                    color: Color::hex("EDEDED").unwrap(),
                }),
                transform: Transform::from_xyz(0.0, 0.0, 1.0),
                ..Default::default()
            },
            item: Item::Armor,
            name: Name(String::from("armor")),
        }
    }
}
