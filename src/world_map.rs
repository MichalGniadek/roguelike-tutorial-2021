use bevy::prelude::*;
use bitflags::bitflags;
use std::ops::{Index, IndexMut};

pub struct Grid {
    pub cell_size: IVec2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

pub struct BlocksMovement;
pub struct BlocksVision;

#[derive(Debug, Clone)]
pub struct Array2D<T> {
    elems: Vec<Vec<T>>,
}

impl<T> Array2D<T> {
    pub fn with_size(x: i32, y: i32) -> Self
    where
        T: Default + Clone,
    {
        Self {
            elems: vec![vec![Default::default(); y as usize]; x as usize],
        }
    }

    pub fn from_vecs(elems: Vec<Vec<T>>) -> Self {
        Self { elems }
    }

    pub fn get(&self, x: i32, y: i32) -> Option<&T> {
        self.elems
            .get(x as usize)
            .map(|r| r.get(y as usize))
            .flatten()
    }

    pub fn get_mut(&mut self, x: i32, y: i32) -> Option<&mut T> {
        self.elems
            .get_mut(x as usize)
            .map(|r| r.get_mut(y as usize))
            .flatten()
    }
}

impl<T> Index<[i32; 2]> for Array2D<T> {
    type Output = T;

    fn index(&self, index: [i32; 2]) -> &Self::Output {
        &self.elems[index[0] as usize][index[1] as usize]
    }
}

impl<T> IndexMut<[i32; 2]> for Array2D<T> {
    fn index_mut(&mut self, index: [i32; 2]) -> &mut Self::Output {
        &mut self.elems[index[0] as usize][index[1] as usize]
    }
}

impl<T> Index<GridPosition> for Array2D<T> {
    type Output = T;

    fn index(&self, index: GridPosition) -> &Self::Output {
        &self.elems[index.x as usize][index.y as usize]
    }
}

impl<T> IndexMut<GridPosition> for Array2D<T> {
    fn index_mut(&mut self, index: GridPosition) -> &mut Self::Output {
        &mut self.elems[index.x as usize][index.y as usize]
    }
}

bitflags! {
    pub struct TileFlags: u32 {
        const BLOCKS_MOVEMENT = 0b00000001;
        const BLOCKS_VISION = 0b00000010;
        const IN_VIEW = 0b00000100;
    }
}

impl Default for TileFlags {
    fn default() -> Self {
        TileFlags::empty()
    }
}

pub struct WorldMap {
    pub world_size: IVec2,
    pub entities: Array2D<Vec<Entity>>,
    pub tile_factory: TileFactory,

    pub tiles: Array2D<TileFlags>,
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

    pub fn wall(&self, commands: &mut Commands, x: i32, y: i32) -> Entity {
        commands
            .spawn_bundle(SpriteBundle {
                material: self.wall_material.clone(),
                ..Default::default()
            })
            .insert_bundle((GridPosition { x, y }, BlocksMovement, BlocksVision))
            .id()
    }

    pub fn floor(&self, commands: &mut Commands, x: i32, y: i32) -> Entity {
        commands
            .spawn_bundle(SpriteBundle {
                material: self.floor_material.clone(),
                ..Default::default()
            })
            .insert(GridPosition { x, y })
            .id()
    }
}
