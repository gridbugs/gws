extern crate rand;
#[macro_use]
extern crate serde;

use rand::Rng;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Input {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Serialize, Deserialize)]
pub struct Cherenkov {}

impl Cherenkov {
    pub fn new<R: Rng>(rng: &mut R) -> Self {
        Self {}
    }
    pub fn tick<I: IntoIterator<Item = Input>, R: Rng>(&mut self, inputs: I, rng: &mut R) {}
}
