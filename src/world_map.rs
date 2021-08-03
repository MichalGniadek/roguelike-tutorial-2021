use crate::dungeon_crawl::Name;
use bevy::{math::ivec2, prelude::*};
use bitflags::bitflags;
use pathfinding::directed::astar;
use std::ops::{Index, IndexMut};

pub struct Grid {
    pub cell_size: IVec2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

pub struct Tile;
pub struct BlocksMovement;
pub struct BlocksVision;

#[derive(Debug, Clone)]
pub struct Array2D<T> {
    elems: Vec<Vec<T>>,
}

impl<T> Array2D<T> {
    pub fn with_elem(x: i32, y: i32, val: T) -> Self
    where
        T: Clone,
    {
        Self {
            elems: vec![vec![val; y as usize]; x as usize],
        }
    }

    pub fn with_size(x: i32, y: i32) -> Self
    where
        T: Default + Clone,
    {
        Self {
            elems: vec![vec![Default::default(); y as usize]; x as usize],
        }
    }

    pub fn _from_vecs(elems: Vec<Vec<T>>) -> Self {
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

    pub fn size(&self) -> IVec2 {
        ivec2(self.elems.len() as i32, self.elems[0].len() as i32)
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

impl<T> Index<[usize; 2]> for Array2D<T> {
    type Output = T;

    fn index(&self, index: [usize; 2]) -> &Self::Output {
        &self.elems[index[0]][index[1]]
    }
}

impl<T> IndexMut<[usize; 2]> for Array2D<T> {
    fn index_mut(&mut self, index: [usize; 2]) -> &mut Self::Output {
        &mut self.elems[index[0]][index[1]]
    }
}

impl<T> Index<(i32, i32)> for Array2D<T> {
    type Output = T;

    fn index(&self, index: (i32, i32)) -> &Self::Output {
        &self.elems[index.0 as usize][index.1 as usize]
    }
}

impl<T> IndexMut<(i32, i32)> for Array2D<T> {
    fn index_mut(&mut self, index: (i32, i32)) -> &mut Self::Output {
        &mut self.elems[index.0 as usize][index.1 as usize]
    }
}

impl<T> Index<IVec2> for Array2D<T> {
    type Output = T;

    fn index(&self, index: IVec2) -> &Self::Output {
        &self.elems[index.x as usize][index.y as usize]
    }
}

impl<T> IndexMut<IVec2> for Array2D<T> {
    fn index_mut(&mut self, index: IVec2) -> &mut Self::Output {
        &mut self.elems[index.x as usize][index.y as usize]
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
        const EXPLORED = 0b00001000;
        const BLOCKS_PATHFINDING = 0b00010000;
    }
}

impl Default for TileFlags {
    fn default() -> Self {
        TileFlags::empty()
    }
}

pub struct WorldMap {
    pub entities: Array2D<Vec<Entity>>,
    pub tile_factory: TileFactory,

    pub tiles: Array2D<TileFlags>,
    pub stairs: GridPosition,
}

impl WorldMap {
    pub fn pathfind(
        &self,
        start: GridPosition,
        end: GridPosition,
    ) -> Option<(Vec<GridPosition>, i32)> {
        astar::astar(
            &start,
            |&GridPosition { x, y }| {
                let mut v = vec![];
                for (i, j) in [(0, 1), (1, 0), (-1, 0), (0, -1)] {
                    if let Some(t) = self.tiles.get(x + i, y + j) {
                        if !t.contains(TileFlags::BLOCKS_PATHFINDING) {
                            let cost = if t.contains(TileFlags::BLOCKS_MOVEMENT) {
                                5
                            } else {
                                1
                            };
                            v.push((GridPosition { x: x + i, y: y + j }, cost));
                        }
                    }
                }
                v
            },
            |&GridPosition { x, y }| {
                f32::sqrt(((end.x - x).pow(2) + (end.y - y).pow(2)) as f32).floor() as i32
            },
            |&pos| pos == end,
        )
    }

    pub fn _line_of_sight(&self, start: GridPosition, end: GridPosition) -> bool {
        let mut previous = None;
        for (x, y) in line_drawing::Bresenham::new((start.x, start.y), (end.x, end.y)) {
            if let Some(&tile) = self.tiles.get(x, y) {
                // Don't go through diagonal walls.
                if let Some((prev_x, prev_y)) = previous {
                    if (self.tiles[[prev_x, y]] & self.tiles[[x, prev_y]])
                        .contains(TileFlags::BLOCKS_VISION)
                    {
                        return false;
                    }
                }

                previous = Some((x, y));

                if tile.contains(TileFlags::BLOCKS_VISION) {
                    return false;
                }
            }
        }
        true
    }
}

pub struct TileFactory {
    pub visible_wall_material: Handle<ColorMaterial>,
    pub visible_floor_material: Handle<ColorMaterial>,
    pub visible_stairs_material: Handle<ColorMaterial>,

    pub explored_wall_material: Handle<ColorMaterial>,
    pub explored_floor_material: Handle<ColorMaterial>,
    pub explored_stairs_material: Handle<ColorMaterial>,
}

impl TileFactory {
    pub fn new(
        asset_server: &Res<AssetServer>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> Self {
        Self {
            visible_wall_material: materials.add(ColorMaterial {
                texture: Some(asset_server.load("brick-wall.png")),
                color: Color::hex("826007").unwrap(),
            }),
            visible_floor_material: materials.add(ColorMaterial {
                texture: Some(asset_server.load("square.png")),
                color: Color::hex("826007").unwrap(),
            }),
            visible_stairs_material: materials.add(ColorMaterial {
                texture: Some(asset_server.load("stairs.png")),
                color: Color::hex("826007").unwrap(),
            }),
            explored_wall_material: materials.add(ColorMaterial {
                texture: Some(asset_server.load("brick-wall.png")),
                color: Color::hex("444444").unwrap(),
            }),
            explored_floor_material: materials.add(ColorMaterial {
                texture: Some(asset_server.load("square.png")),
                color: Color::hex("444444").unwrap(),
            }),
            explored_stairs_material: materials.add(ColorMaterial {
                texture: Some(asset_server.load("stairs.png")),
                color: Color::hex("444444").unwrap(),
            }),
        }
    }

    pub fn wall(&self, commands: &mut Commands, x: i32, y: i32) -> Entity {
        commands
            .spawn_bundle(SpriteBundle {
                material: self.explored_wall_material.clone(),
                ..Default::default()
            })
            .insert_bundle((
                Tile,
                GridPosition { x, y },
                BlocksMovement,
                BlocksVision,
                Name(String::from("wall")),
            ))
            .id()
    }

    pub fn floor(&self, commands: &mut Commands, x: i32, y: i32) -> Entity {
        commands
            .spawn_bundle(SpriteBundle {
                material: self.explored_floor_material.clone(),
                ..Default::default()
            })
            .insert_bundle((Tile, GridPosition { x, y }, Name(String::from("floor"))))
            .id()
    }

    pub fn stairs(&self, commands: &mut Commands, x: i32, y: i32) -> Entity {
        commands
            .spawn_bundle(SpriteBundle {
                material: self.explored_stairs_material.clone(),
                ..Default::default()
            })
            .insert_bundle((Tile, GridPosition { x, y }, Name(String::from("stairs"))))
            .id()
    }
}
