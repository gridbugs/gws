use coord_2d::*;
use direction::*;
use grid_2d::*;
use hashbrown::{hash_set, HashMap, HashSet};
use rgb24::*;
use shadowcast::*;

pub enum Instruction {
    SetBackground(Coord, BackgroundTile),
    AddEntity(Coord, PackedEntity),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Rational {
    pub num: u32,
    pub denom: u32,
}

impl Rational {
    pub fn new(num: u32, denom: u32) -> Self {
        Self { num, denom }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackgroundTile {
    Floor,
    Ground,
    IceWall,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForegroundTile {
    Player,
    Tree,
    Stairs,
    Demon,
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
        loop {
            if let Some(e) = self.0.next() {
                if let Some(foreground_tile) = e.foreground_tile() {
                    return Some(foreground_tile);
                }
            } else {
                return None;
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Light {
    coord: Coord,
    colour: Rgb24,
    range: vision_distance::Circle,
    diminish: Rational,
}

impl Light {
    fn new(coord: Coord, colour: Rgb24, range_squared: u32, diminish: Rational) -> Self {
        Self {
            coord,
            colour,
            range: vision_distance::Circle::new_squared(range_squared),
            diminish,
        }
    }
    fn diminish_at_coord(&self, coord: Coord) -> u32 {
        ((self.coord - coord).magnitude2() * self.diminish.num / self.diminish.denom)
            .max(1)
    }
    pub(crate) fn colour_at_coord(&self, coord: Coord) -> Rgb24 {
        self.colour.scalar_div(self.diminish_at_coord(coord))
    }
    pub(crate) fn range(&self) -> vision_distance::Circle {
        self.range
    }
    pub(crate) fn coord(&self) -> Coord {
        self.coord
    }
    fn pack(&self) -> PackedLight {
        PackedLight {
            colour: self.colour,
            range_squared: self.range.distance_squared(),
            diminish: self.diminish,
        }
    }
}

pub type EntityId = u64;

#[derive(Serialize, Deserialize)]
pub struct Entity {
    coord: Coord,
    foreground_tile: Option<ForegroundTile>,
    light_index: Option<usize>,
}

impl Entity {
    pub fn coord(&self) -> Coord {
        self.coord
    }
    pub fn foreground_tile(&self) -> Option<ForegroundTile> {
        self.foreground_tile
    }
}

#[derive(Clone)]
pub struct PackedEntity {
    pub(crate) foreground_tile: Option<ForegroundTile>,
    pub(crate) light: Option<PackedLight>,
    pub(crate) npc: bool,
}

impl Default for PackedEntity {
    fn default() -> Self {
        Self {
            foreground_tile: None,
            light: None,
            npc: false,
        }
    }
}

impl PackedEntity {
    pub(crate) fn player() -> Self {
        let player_light = PackedLight::new(grey24(128), 30, Rational::new(1, 10));
        Self {
            foreground_tile: Some(ForegroundTile::Player),
            light: Some(player_light),
            npc: false,
        }
    }
    pub(crate) fn demon() -> Self {
        Self {
            foreground_tile: Some(ForegroundTile::Demon),
            light: None,
            npc: true,
        }
    }
}

#[derive(Clone)]
pub struct PackedLight {
    pub colour: Rgb24,
    pub range_squared: u32,
    pub diminish: Rational,
}

impl PackedLight {
    pub fn new(colour: Rgb24, range_squared: u32, diminish: Rational) -> Self {
        Self {
            colour,
            range_squared,
            diminish,
        }
    }
    pub fn light(self, coord: Coord) -> Light {
        let PackedLight {
            colour,
            range_squared,
            diminish,
        } = self;
        Light::new(coord, colour, range_squared, diminish)
    }
}

#[derive(Serialize, Deserialize)]
pub struct WorldCell {
    background_tile: BackgroundTile,
    entities: HashSet<EntityId>,
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
    pub fn entity_iter<'a>(&'a self, entities: &'a Entities) -> EntityIter<'a> {
        EntityIter {
            iter: self.entities.iter(),
            entities,
        }
    }
    pub fn foreground_tiles<'a>(&'a self, entities: &'a Entities) -> ForegroundTiles<'a> {
        ForegroundTiles(self.entity_iter(entities))
    }
    pub fn is_solid(&self) -> bool {
        self.background_tile == BackgroundTile::IceWall
    }
}

impl Default for WorldCell {
    fn default() -> Self {
        Self::new(BackgroundTile::Floor)
    }
}

pub type Entities = HashMap<EntityId, Entity>;

#[derive(Serialize, Deserialize)]
pub struct World {
    grid: Grid<WorldCell>,
    lights: Vec<Light>,
    entities: HashMap<EntityId, Entity>,
    next_id: EntityId,
    npc_ids: HashSet<EntityId>,
}

impl World {
    pub(crate) fn new(size: Size) -> Self {
        Self {
            grid: Grid::new_default(size),
            lights: Vec::new(),
            entities: HashMap::new(),
            next_id: 0,
            npc_ids: HashSet::new(),
        }
    }
    pub(crate) fn pack_entity(&self, id: EntityId) -> PackedEntity {
        let entity = self.entities.get(&id).unwrap();
        PackedEntity {
            foreground_tile: entity.foreground_tile,
            light: entity.light_index.map(|index| self.lights[index].pack()),
            npc: self.npc_ids.contains(&id),
        }
    }
    pub(crate) fn lights(&self) -> &[Light] {
        &self.lights
    }
    pub fn grid(&self) -> &Grid<WorldCell> {
        &self.grid
    }
    pub fn entities(&self) -> &Entities {
        &self.entities
    }
    pub(crate) fn add_entity(&mut self, coord: Coord, entity: PackedEntity) -> EntityId {
        let PackedEntity {
            foreground_tile,
            light,
            npc,
        } = entity;
        let id = self.next_id;
        self.next_id += 1;
        let light_index = light.map(|packed_light| {
            let light_index = self.lights.len();
            self.lights.push(packed_light.light(coord));
            light_index
        });
        let entity = Entity {
            coord,
            foreground_tile,
            light_index,
        };
        self.entities.insert(id, entity);
        if let Some(cell) = self.grid.get_mut(coord) {
            cell.entities.insert(id);
        }
        if npc {
            self.npc_ids.insert(id);
        }
        id
    }
    fn set_background(&mut self, coord: Coord, background_tile: BackgroundTile) {
        let cell = self.grid.get_checked_mut(coord);
        cell.background_tile = background_tile;
    }
    pub(crate) fn interpret_instruction(&mut self, instruction: Instruction) {
        use Instruction::*;
        match instruction {
            SetBackground(coord, background_tile) => {
                self.set_background(coord, background_tile)
            }
            AddEntity(coord, packed_entity) => {
                self.add_entity(coord, packed_entity);
            }
        }
    }
    pub(crate) fn npc_ids(&self) -> impl Iterator<Item = &EntityId> {
        self.npc_ids.iter()
    }
    pub(crate) fn move_entity_in_direction(
        &mut self,
        id: EntityId,
        direction: CardinalDirection,
    ) -> Coord {
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
        next_coord
    }
    pub(crate) fn opacity(&self, coord: Coord) -> u8 {
        let cell = self.grid.get_checked(coord);
        let background = match cell.background_tile {
            BackgroundTile::Floor => 0,
            BackgroundTile::Ground => 0,
            BackgroundTile::IceWall => 128,
        };
        let foreground = cell
            .entity_iter(&self.entities)
            .filter_map(|e| {
                e.foreground_tile()
                    .map(|foreground_tile| match foreground_tile {
                        ForegroundTile::Player => 0,
                        ForegroundTile::Stairs => 0,
                        ForegroundTile::Demon => 0,
                        ForegroundTile::Tree => 128,
                    })
            })
            .max()
            .unwrap_or(0);
        background.max(foreground)
    }
}
