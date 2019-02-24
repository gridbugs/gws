extern crate coord_2d;
extern crate direction;
extern crate grid_2d;
extern crate rand;
#[macro_use]
extern crate serde;
extern crate shadowcast;

use coord_2d::{Coord, Size};
use direction::CardinalDirection;
use grid_2d::Grid;
use rand::Rng;
use shadowcast::*;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Cell {
    Floor,
    Wall,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Input {
    Move(CardinalDirection),
}

pub mod input {
    use super::*;
    pub const UP: Input = Input::Move(CardinalDirection::North);
    pub const DOWN: Input = Input::Move(CardinalDirection::South);
    pub const LEFT: Input = Input::Move(CardinalDirection::West);
    pub const RIGHT: Input = Input::Move(CardinalDirection::East);
}

struct Visibility;

impl InputGrid for Visibility {
    type Grid = Grid<Cell>;
    type Opacity = u8;
    fn size(&self, grid: &Self::Grid) -> Size {
        grid.size()
    }
    fn get_opacity(&self, grid: &Self::Grid, coord: Coord) -> Self::Opacity {
        match *grid.get_checked(coord) {
            Cell::Floor => 0,
            Cell::Wall => 255,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Cherenkov {
    grid: Grid<Cell>,
    player_coord: Coord,
    #[serde(skip)]
    shadowcast: ShadowcastContext<u8>,
}

impl Cherenkov {
    pub fn new<R: Rng>(rng: &mut R) -> Self {
        let _ = rng;
        let terrain_vecs = include_str!("terrain_strings.txt")
            .split("\n")
            .filter(|s| !s.is_empty())
            .map(|s| s.chars().collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let size = Size::new(terrain_vecs[0].len() as u32, terrain_vecs.len() as u32);
        let mut player_coord = Coord::new(0, 0);
        let grid = Grid::new_fn(size, |coord| {
            match terrain_vecs[coord.y as usize][coord.x as usize] {
                '.' => Cell::Floor,
                '#' => Cell::Wall,
                '@' => {
                    player_coord = coord;
                    Cell::Floor
                }
                _ => panic!(),
            }
        });
        let shadowcast = ShadowcastContext::default();
        Self {
            grid,
            player_coord,
            shadowcast,
        }
    }

    pub fn tick<I: IntoIterator<Item = Input>, R: Rng>(&mut self, inputs: I, rng: &mut R) {
        let _ = rng;
        for i in inputs {
            match i {
                Input::Move(direction) => self.player_coord += direction.coord(),
            }
        }
    }

    pub fn player_coord(&self) -> Coord {
        self.player_coord
    }

    pub fn grid(&self) -> &Grid<Cell> {
        &self.grid
    }
}
