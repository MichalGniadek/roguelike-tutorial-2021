use bevy::prelude::*;

pub struct Grid {
    pub cell_size: IVec2,
}

#[derive(Debug, Clone, Copy)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

pub struct Tile {
    pub walkable: bool,
}

#[derive(Bundle)]
pub struct TileBundle {
    #[bundle]
    sprite: SpriteBundle,
    position: GridPosition,
    tile: Tile,
}

pub struct WorldMap {
    pub world_size: IVec2,
    pub tiles: Vec<Vec<Option<Entity>>>,
    pub tile_factory: TileFactory,
}

pub struct TileFactory {
    wall_material: Handle<ColorMaterial>,
    floor_material: Handle<ColorMaterial>,
}

impl TileFactory {
    pub fn new(
        asset_server: &Res<AssetServer>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> Self {
        Self {
            wall_material: materials.add(ColorMaterial {
                texture: Some(asset_server.load("brick-wall.png")),
                color: Color::hex("444444").unwrap(),
            }),
            floor_material: materials.add(ColorMaterial {
                texture: Some(asset_server.load("square.png")),
                color: Color::hex("444444").unwrap(),
            }),
        }
    }

    pub fn wall(&self, x: i32, y: i32) -> TileBundle {
        TileBundle {
            sprite: SpriteBundle {
                material: self.wall_material.clone(),
                ..Default::default()
            },
            position: GridPosition { x, y },
            tile: Tile { walkable: false },
        }
    }

    pub fn floor(&self, x: i32, y: i32) -> TileBundle {
        TileBundle {
            sprite: SpriteBundle {
                material: self.floor_material.clone(),
                ..Default::default()
            },
            position: GridPosition { x, y },
            tile: Tile { walkable: true },
        }
    }
}
