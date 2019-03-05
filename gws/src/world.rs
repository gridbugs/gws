use super::Animation;
use coord_2d::*;
use direction::*;
use grid_2d::*;
use hashbrown::{hash_set, HashMap, HashSet};
use line_2d::*;
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
    id: EntityId,
    coord: Coord,
    foreground_tile: Option<ForegroundTile>,
    light_index: Option<usize>,
    npc: bool,
    player: bool,
    taking_damage_in_direction: Option<CardinalDirection>,
    hit_points: Option<Rational>,
}

impl Entity {
    pub fn coord(&self) -> Coord {
        self.coord
    }
    pub fn foreground_tile(&self) -> Option<ForegroundTile> {
        self.foreground_tile
    }
    pub fn taking_damage_in_direction(&self) -> Option<CardinalDirection> {
        self.taking_damage_in_direction
    }
    pub fn hit_points(&self) -> Option<Rational> {
        self.hit_points
    }
    pub fn is_npc(&self) -> bool {
        self.npc
    }
}

#[derive(Clone)]
pub struct PackedEntity {
    pub(crate) foreground_tile: Option<ForegroundTile>,
    pub(crate) light: Option<PackedLight>,
    pub(crate) npc: bool,
    pub(crate) player: bool,
    pub(crate) hit_points: Option<Rational>,
}

impl Default for PackedEntity {
    fn default() -> Self {
        Self {
            foreground_tile: None,
            light: None,
            npc: false,
            player: false,
            hit_points: None,
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
            player: true,
            hit_points: Some(Rational::new(1, 4)),
        }
    }
    pub(crate) fn demon() -> Self {
        Self {
            foreground_tile: Some(ForegroundTile::Demon),
            light: None,
            npc: true,
            player: false,
            hit_points: Some(Rational::new(2, 2)),
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

#[derive(Debug, Serialize, Deserialize)]
pub struct WorldCell {
    background_tile: BackgroundTile,
    entities: HashSet<EntityId>,
    npc_count: usize,
    player_count: usize,
}

impl WorldCell {
    fn new(background_tile: BackgroundTile) -> Self {
        Self {
            background_tile,
            entities: HashSet::new(),
            npc_count: 0,
            player_count: 0,
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
    pub fn contains_npc(&self) -> bool {
        self.npc_count > 0
    }
    pub fn contains_player(&self) -> bool {
        self.player_count > 0
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

#[derive(Debug)]
pub enum CancelAction {
    MoveIntoSolidCell,
    NpcMoveIntoNpc,
    MoveOutOfBounds,
    NoEntity,
}

pub(crate) enum ApplyAction {
    Done,
    Animation(Animation),
}

fn move_entity_to_coord(
    coord: Coord,
    entity: &mut Entity,
    grid: &mut Grid<WorldCell>,
    lights: &mut Vec<Light>,
) {
    if let Some(current_cell) = grid.get_mut(entity.coord) {
        current_cell.entities.remove(&entity.id);
        if entity.npc {
            current_cell.npc_count -= 1;
        }
        if entity.player {
            current_cell.player_count -= 1;
        }
    }
    if let Some(next_cell) = grid.get_mut(coord) {
        next_cell.entities.insert(entity.id);
        if entity.npc {
            next_cell.npc_count += 1;
        }
        if entity.player {
            next_cell.player_count += 1;
        }
    }
    entity.coord = coord;
    if let Some(light_index) = entity.light_index {
        let light = lights.get_mut(light_index).unwrap();
        light.coord = entity.coord;
    }
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
            npc: entity.npc,
            player: entity.player,
            hit_points: entity.hit_points,
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
            player,
            hit_points,
        } = entity;
        let id = self.next_id;
        self.next_id += 1;
        let light_index = light.map(|packed_light| {
            let light_index = self.lights.len();
            self.lights.push(packed_light.light(coord));
            light_index
        });
        let entity = Entity {
            id,
            coord,
            foreground_tile,
            light_index,
            npc,
            player,
            taking_damage_in_direction: None,
            hit_points,
        };
        self.entities.insert(id, entity);
        if let Some(cell) = self.grid.get_mut(coord) {
            cell.entities.insert(id);
            if npc {
                cell.npc_count += 1;
            }
            if player {
                cell.player_count += 1;
            }
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
    ) -> Result<ApplyAction, CancelAction> {
        if let Some(entity) = self.entities.get_mut(&id) {
            let coord = entity.coord + direction.coord();
            if let Some(cell) = self.grid.get(coord) {
                if cell.is_solid() {
                    Err(CancelAction::MoveIntoSolidCell)
                } else if entity.npc && cell.contains_npc() {
                    Err(CancelAction::NpcMoveIntoNpc)
                } else if entity.npc && cell.contains_player() {
                    let id = cell
                        .entity_iter(&self.entities)
                        .find_map(|e| if e.player { Some(e.id) } else { None })
                        .unwrap();
                    Ok(ApplyAction::Animation(Animation::damage(id, direction)))
                } else if entity.player && cell.contains_npc() {
                    let id = cell
                        .entity_iter(&self.entities)
                        .find_map(|e| if e.npc { Some(e.id) } else { None })
                        .unwrap();
                    Ok(ApplyAction::Animation(Animation::damage(id, direction)))
                } else {
                    move_entity_to_coord(coord, entity, &mut self.grid, &mut self.lights);
                    Ok(ApplyAction::Done)
                }
            } else {
                Err(CancelAction::MoveOutOfBounds)
            }
        } else {
            Err(CancelAction::NoEntity)
        }
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
    pub(crate) fn can_see(&self, a: Coord, b: Coord, max_distance: usize) -> bool {
        let mut visibility = 255u8;
        let line_segment = LineSegment::new(a, b);
        if line_segment.num_steps() > max_distance {
            return false;
        }
        for coord in line_segment {
            visibility = visibility.saturating_sub(self.opacity(coord));
        }
        visibility > 0
    }
    pub(crate) fn set_taking_damage_in_direction(
        &mut self,
        id: EntityId,
        value: Option<CardinalDirection>,
    ) {
        self.entities
            .get_mut(&id)
            .unwrap()
            .taking_damage_in_direction = value;
    }
    pub(crate) fn deal_damage(&mut self, id: EntityId, damage: u32) {
        if let Some(entity) = self.entities.get_mut(&id) {
            if let Some(hit_points) = entity.hit_points.as_mut() {
                hit_points.num = hit_points.num.saturating_sub(damage);
                if hit_points.num == 0 {
                    self.remove_entity(id);
                }
            }
        }
    }
    pub(crate) fn remove_entity(&mut self, id: EntityId) {
        if let Some(entity) = self.entities.get(&id) {
            if entity.player {
                return;
            }
        }
        if let Some(entity) = self.entities.remove(&id) {
            if entity.npc {
                self.npc_ids.remove(&id);
            }
            if let Some(cell) = self.grid.get_mut(entity.coord) {
                cell.entities.remove(&id);
                if entity.npc {
                    cell.npc_count -= 1;
                }
            }
        }
    }
}
