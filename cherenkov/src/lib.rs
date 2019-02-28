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
    type Grid = Grid<WorldCell>;
    type Opacity = u8;
    fn size(&self, grid: &Self::Grid) -> Size {
        grid.size()
    }
    fn get_opacity(&self, grid: &Self::Grid, coord: Coord) -> Self::Opacity {
        match grid.get_checked(coord).base() {
            WorldCellBase::Floor => 0,
            WorldCellBase::Wall => 255,
        }
    }
}

const VISION_DISTANCE: vision_distance::Circle = vision_distance::Circle::new(8);

#[derive(Clone, Serialize, Deserialize)]
pub struct VisibilityCell {
    last_seen: u64,
}

#[derive(Clone, Copy)]
pub struct VisibilityState {
    count: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VisibileArea {
    grid: Grid<VisibilityCell>,
    count: u64,
    #[serde(skip)]
    shadowcast: ShadowcastContext<u8>,
}

impl VisibileArea {
    pub fn new(size: Size) -> Self {
        let grid = Grid::new_clone(size, VisibilityCell { last_seen: 0 });
        let count = 1;
        let shadowcast = ShadowcastContext::default();
        Self {
            grid,
            count,
            shadowcast,
        }
    }
    pub fn state(&self) -> VisibilityState {
        VisibilityState { count: self.count }
    }
    pub fn iter(&self) -> impl Iterator<Item = &VisibilityCell> {
        self.grid.iter()
    }
    pub fn update(&mut self, player_coord: Coord, world: &Grid<WorldCell>) {
        self.count += 1;
        let count = self.count;
        let grid = &mut self.grid;
        self.shadowcast.for_each_visible(
            player_coord,
            &Visibility,
            &world,
            VISION_DISTANCE,
            255,
            |coord, direction_bitmap, visibility| {
                grid.get_checked_mut(coord).last_seen = count;
            },
        );
    }
}

impl VisibilityCell {
    pub fn is_visible(&self, state: VisibilityState) -> bool {
        self.last_seen == state.count
    }
    pub fn is_discovered(&self) -> bool {
        self.last_seen != 0
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum WorldCellBase {
    Floor,
    Wall,
}

#[derive(Serialize, Deserialize)]
pub struct WorldCell {
    base: WorldCellBase,
}

impl WorldCell {
    fn new(base: WorldCellBase) -> Self {
        Self { base }
    }
    pub fn base(&self) -> WorldCellBase {
        self.base
    }
}

#[derive(Serialize, Deserialize)]
pub struct World {
    grid: Grid<WorldCell>,
}

impl World {
    pub fn grid(&self) -> &Grid<WorldCell> {
        &self.grid
    }
}

#[derive(Serialize, Deserialize)]
pub struct Cherenkov {
    world: World,
    visible_area: VisibileArea,
    player_coord: Coord,
}

pub struct ToRender<'a> {
    pub world: &'a World,
    pub visible_area: &'a VisibileArea,
    pub player_coord: &'a Coord,
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
            let base = match terrain_vecs[coord.y as usize][coord.x as usize] {
                '.' => WorldCellBase::Floor,
                '#' => WorldCellBase::Wall,
                '@' => {
                    player_coord = coord;
                    WorldCellBase::Floor
                }
                _ => panic!(),
            };
            WorldCell::new(base)
        });
        let world = World { grid };
        let mut visible_area = VisibileArea::new(size);
        visible_area.update(player_coord, world.grid());
        Self {
            world,
            visible_area,
            player_coord,
        }
    }

    pub fn tick<I: IntoIterator<Item = Input>, R: Rng>(&mut self, inputs: I, rng: &mut R) {
        let _ = rng;
        for i in inputs {
            match i {
                Input::Move(direction) => self.player_coord += direction.coord(),
            }
        }
        self.visible_area
            .update(self.player_coord, &self.world.grid());
    }

    pub fn to_render(&self) -> ToRender {
        ToRender {
            world: &self.world,
            visible_area: &self.visible_area,
            player_coord: &self.player_coord,
        }
    }
}
