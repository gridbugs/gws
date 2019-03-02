extern crate coord_2d;
extern crate direction;
extern crate grid_2d;
extern crate rand;
#[macro_use]
extern crate serde;
extern crate hashbrown;
extern crate rgb24;
extern crate shadowcast;
extern crate wfc;

mod terrain;
mod vision;
mod world;

use crate::vision::*;
pub use crate::world::*;
use direction::*;
use rand::Rng;

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
    world: World,
    visible_area: VisibileArea,
}

pub struct ToRender<'a> {
    pub world: &'a World,
    pub visible_area: &'a VisibileArea,
    pub player: &'a Entity,
}

impl Cherenkov {
    pub fn new<R: Rng>(rng: &mut R) -> Self {
        let _ = rng;
        let (size, instructions) = terrain::from_str(include_str!("terrain_string.txt"));
        let mut world = World::new(size);
        for instruction in instructions {
            world.interpret_instruction(instruction);
        }
        let visible_area = VisibileArea::new(size);
        let mut s = Self {
            world,
            visible_area,
        };
        s.update_visible_area();
        s
    }

    pub fn tick<I: IntoIterator<Item = Input>, R: Rng>(
        &mut self,
        inputs: I,
        rng: &mut R,
    ) {
        let player_id = self.world.player_id();
        let _ = rng;
        for i in inputs {
            match i {
                Input::Move(direction) => {
                    self.world.move_entity_in_direction(player_id, direction)
                }
            }
        }
        self.update_visible_area();
    }

    fn update_visible_area(&mut self) {
        self.visible_area
            .update(self.world.player().coord(), &self.world);
    }

    pub fn to_render(&self) -> ToRender {
        ToRender {
            world: &self.world,
            visible_area: &self.visible_area,
            player: &self.world.player(),
        }
    }
}
