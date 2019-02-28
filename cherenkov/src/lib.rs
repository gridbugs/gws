extern crate coord_2d;
extern crate direction;
extern crate grid_2d;
extern crate rand;
#[macro_use]
extern crate serde;
extern crate hashbrown;
extern crate rgb24;
extern crate shadowcast;

use coord_2d::{Coord, Size};
use direction::*;
use grid_2d::Grid;
use hashbrown::{hash_set, HashMap, HashSet};
use rand::Rng;
use rgb24::*;
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
        grid.get_checked(coord).opacity()
    }
}

const VISION_DISTANCE_SCALAR: u32 = 12;
const VISION_DISTANCE: vision_distance::Circle =
    vision_distance::Circle::new(VISION_DISTANCE_SCALAR);
const PLAYER_LIGHT_DISTANCE_SCALAR: u32 = 6;

#[derive(Clone, Serialize, Deserialize)]
pub struct VisibilityCell {
    last_seen: u64,
    last_lit: u64,
    visible_directions: DirectionBitmap,
    light_colour: Rgb24,
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
        let grid = Grid::new_clone(
            size,
            VisibilityCell {
                last_seen: 0,
                last_lit: 0,
                light_colour: rgb24(0, 0, 0),
                visible_directions: DirectionBitmap::empty(),
            },
        );
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
    pub fn update(&mut self, player_coord: Coord, world: &World) {
        self.count += 1;
        let count = self.count;
        let grid = &mut self.grid;
        self.shadowcast.for_each_visible(
            player_coord,
            &Visibility,
            world.grid(),
            VISION_DISTANCE,
            255,
            |coord, direction_bitmap, _visibility| {
                let cell = grid.get_checked_mut(coord);
                cell.last_seen = count;
                cell.visible_directions = direction_bitmap;
            },
        );
        for light in world.lights.iter() {
            self.shadowcast.for_each_visible(
                light.coord,
                &Visibility,
                world.grid(),
                light.range,
                255,
                |coord, direction_bitmap, _visibility| {
                    let cell = grid.get_checked_mut(coord);
                    if !(direction_bitmap & cell.visible_directions).is_empty() {
                        if cell.last_lit != count {
                            cell.last_lit = count;
                            cell.light_colour = rgb24(0, 0, 0);
                        }
                        cell.light_colour = cell
                            .light_colour
                            .saturating_add(light.colour_at_coord(coord));
                    }
                },
            );
        }
    }
}

impl VisibilityCell {
    pub fn is_visible(&self, state: VisibilityState) -> bool {
        self.last_seen == state.count
    }
    pub fn is_discovered(&self) -> bool {
        self.last_seen != 0
    }
    pub fn light_colour(&self, state: VisibilityState) -> Rgb24 {
        if self.last_lit == state.count {
            self.light_colour
        } else {
            rgb24(0, 0, 0)
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum BackgroundTile {
    Floor,
    Wall,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ForegroundTile {
    Player,
}

#[derive(Serialize, Deserialize)]
pub struct WorldCell {
    background_tile: BackgroundTile,
    entities: HashSet<EntityId>,
}

pub struct EntityIter<'a> {
    iter: hash_set::Iter<'a, EntityId>,
    entities: &'a Entities,
}

impl<'a> Iterator for EntityIter<'a> {
    type Item = &'a Entity;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|id| self.entities.get(id).unwrap())
    }
}

pub struct ForegroundTiles<'a>(EntityIter<'a>);

impl<'a> Iterator for ForegroundTiles<'a> {
    type Item = ForegroundTile;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|e| e.foreground_tile)
    }
}

impl WorldCell {
    fn new(background_tile: BackgroundTile) -> Self {
        Self {
            background_tile,
            entities: HashSet::new(),
        }
    }
    pub fn background_tile(&self) -> BackgroundTile {
        self.background_tile
    }
    pub fn opacity(&self) -> u8 {
        match self.background_tile {
            BackgroundTile::Floor => 0,
            BackgroundTile::Wall => 255,
        }
    }
    pub fn entity_iter<'a>(&'a self, entities: &'a Entities) -> EntityIter<'a> {
        EntityIter {
            iter: self.entities.iter(),
            entities,
        }
    }
    pub fn foreground_tiles<'a>(&'a self, entities: &'a Entities) -> ForegroundTiles<'a> {
        ForegroundTiles(self.entity_iter(entities))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Light {
    coord: Coord,
    colour: Rgb24,
    range: vision_distance::Circle,
    diminish_num: u32,
    diminish_denom: u32,
}

impl Light {
    fn new(
        coord: Coord,
        colour: Rgb24,
        range: u32,
        diminish_num: u32,
        diminish_denom: u32,
    ) -> Self {
        Self {
            coord,
            colour,
            range: vision_distance::Circle::new(range),
            diminish_num,
            diminish_denom,
        }
    }
    fn diminish_at_coord(&self, coord: Coord) -> u32 {
        ((self.coord - coord).magnitude2() * self.diminish_num / self.diminish_denom)
            .max(1)
    }
    fn colour_at_coord(&self, coord: Coord) -> Rgb24 {
        self.colour.scalar_div(self.diminish_at_coord(coord))
    }
}

pub type EntityId = u64;

#[derive(Serialize, Deserialize)]
pub struct Entity {
    coord: Coord,
    foreground_tile: ForegroundTile,
    light_index: Option<usize>,
}

impl Entity {
    pub fn coord(&self) -> Coord {
        self.coord
    }
    pub fn foreground_tile(&self) -> ForegroundTile {
        self.foreground_tile
    }
}

pub type Entities = HashMap<EntityId, Entity>;

#[derive(Serialize, Deserialize)]
pub struct World {
    grid: Grid<WorldCell>,
    lights: Vec<Light>,
    entities: HashMap<EntityId, Entity>,
}

impl World {
    pub fn grid(&self) -> &Grid<WorldCell> {
        &self.grid
    }
    pub fn entities(&self) -> &Entities {
        &self.entities
    }
    pub fn move_entity_in_direction(
        &mut self,
        id: EntityId,
        direction: CardinalDirection,
    ) {
        let entity = self.entities.get_mut(&id).unwrap();
        let next_coord = entity.coord + direction.coord();
        if let Some(current_cell) = self.grid.get_mut(entity.coord) {
            current_cell.entities.remove(&id);
        }
        if let Some(next_cell) = self.grid.get_mut(next_coord) {
            next_cell.entities.insert(id);
        }
        entity.coord = next_coord;
        if let Some(light_index) = entity.light_index {
            let light = self.lights.get_mut(light_index).unwrap();
            light.coord = entity.coord;
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Cherenkov {
    world: World,
    visible_area: VisibileArea,
    player_id: EntityId,
    next_id: EntityId,
}

pub struct ToRender<'a> {
    pub world: &'a World,
    pub visible_area: &'a VisibileArea,
    pub player: &'a Entity,
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
        let mut lights = Vec::new();
        let light_base = 10;
        let grid = Grid::new_fn(size, |coord| {
            let base = match terrain_vecs[coord.y as usize][coord.x as usize] {
                '.' => BackgroundTile::Floor,
                '#' => BackgroundTile::Wall,
                '@' => {
                    player_coord = coord;
                    BackgroundTile::Floor
                }
                '1' => {
                    lights.push(Light::new(
                        coord,
                        rgb24(255, light_base, light_base),
                        10,
                        2,
                        5,
                    ));
                    BackgroundTile::Floor
                }
                '2' => {
                    lights.push(Light::new(
                        coord,
                        rgb24(light_base, 255, light_base),
                        10,
                        3,
                        5,
                    ));
                    BackgroundTile::Floor
                }
                '3' => {
                    lights.push(Light::new(
                        coord,
                        rgb24(light_base, light_base, 255),
                        10,
                        2,
                        5,
                    ));
                    BackgroundTile::Floor
                }
                '4' => {
                    lights.push(Light::new(coord, rgb24(255, 255, light_base), 10, 3, 5));
                    BackgroundTile::Floor
                }
                _ => panic!(),
            };
            WorldCell::new(base)
        });
        let mut world = World {
            grid,
            lights,
            entities: HashMap::new(),
        };
        let mut visible_area = VisibileArea::new(size);
        visible_area.update(player_coord, &world);
        let player_light = Light::new(
            player_coord,
            rgb24(128, 128, 128),
            PLAYER_LIGHT_DISTANCE_SCALAR,
            4,
            5,
        );
        let player = Entity {
            coord: player_coord,
            foreground_tile: ForegroundTile::Player,
            light_index: Some(world.lights.len()),
        };
        world.lights.push(player_light);
        let player_id = 0;
        let next_id = 1;
        world
            .grid
            .get_checked_mut(player_coord)
            .entities
            .insert(player_id);
        world.entities.insert(player_id, player);
        Self {
            world,
            visible_area,
            player_id,
            next_id,
        }
    }

    pub fn tick<I: IntoIterator<Item = Input>, R: Rng>(
        &mut self,
        inputs: I,
        rng: &mut R,
    ) {
        let _ = rng;
        for i in inputs {
            match i {
                Input::Move(direction) => self
                    .world
                    .move_entity_in_direction(self.player_id, direction),
            }
        }
        self.visible_area.update(
            self.world.entities.get(&self.player_id).unwrap().coord,
            &self.world,
        );
    }

    pub fn to_render(&self) -> ToRender {
        ToRender {
            world: &self.world,
            visible_area: &self.visible_area,
            player: &self.world.entities.get(&self.player_id).unwrap(),
        }
    }
}
