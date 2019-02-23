extern crate coord_2d;
extern crate direction;
extern crate grid_2d;
extern crate rand;
#[macro_use]
extern crate serde;

use coord_2d::{Coord, Size};
use direction::CardinalDirection;
use grid_2d::Grid;
use rand::Rng;

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

#[derive(Serialize, Deserialize)]
pub struct Cherenkov {
    grid: Grid<Cell>,
    player: Coord,
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
        let mut player = Coord::new(0, 0);
        let grid = Grid::new_fn(size, |coord| {
            match terrain_vecs[coord.y as usize][coord.x as usize] {
                '.' => Cell::Floor,
                '#' => Cell::Wall,
                '@' => {
                    player = coord;
                    Cell::Floor
                }
                _ => panic!(),
            }
        });
        Self { grid, player }
    }
    pub fn tick<I: IntoIterator<Item = Input>, R: Rng>(&mut self, inputs: I, rng: &mut R) {
        let _ = rng;
        for i in inputs {
            match i {
                Input::Move(direction) => self.player += direction.coord(),
            }
        }
    }
    pub fn player(&self) -> Coord {
        self.player
    }
    pub fn grid(&self) -> &Grid<Cell> {
        &self.grid
    }
}
