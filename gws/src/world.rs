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

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct HitPoints {
    pub current: u32,
    pub max: u32,
}

impl HitPoints {
    fn new(current: u32, max: u32) -> Self {
        Self { current, max }
    }
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
    Block,
    Player,
    Tree,
    Stairs,
    Bumper,
    Caster,
    Healer,
    Blink0,
    Blink1,
    Flame,
    Altar,
    Fountain,
    Spark,
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

pub type LightId = u64;
pub type EntityId = u64;

#[derive(Debug, Serialize, Deserialize)]
pub struct Entity {
    id: EntityId,
    coord: Coord,
    foreground_tile: Option<ForegroundTile>,
    light_index: Option<LightId>,
    npc: bool,
    player: bool,
    interactive: bool,
    solid: bool,
    taking_damage_in_direction: Option<CardinalDirection>,
    hit_points: Option<HitPoints>,
    heal_countdown: Option<u32>,
}

impl Entity {
    pub fn id(&self) -> EntityId {
        self.id
    }
    pub fn coord(&self) -> Coord {
        self.coord
    }
    pub fn foreground_tile(&self) -> Option<ForegroundTile> {
        self.foreground_tile
    }
    pub fn taking_damage_in_direction(&self) -> Option<CardinalDirection> {
        self.taking_damage_in_direction
    }
    pub fn hit_points(&self) -> Option<HitPoints> {
        self.hit_points
    }
    pub fn is_npc(&self) -> bool {
        self.npc
    }
    pub fn is_projectile(&self) -> bool {
        self.foreground_tile == Some(ForegroundTile::Spark)
    }
    pub fn heal_countdown(&self) -> Option<u32> {
        self.heal_countdown
    }
}

#[derive(Clone)]
pub struct PackedEntity {
    pub(crate) foreground_tile: Option<ForegroundTile>,
    pub(crate) light: Option<PackedLight>,
    pub(crate) npc: bool,
    pub(crate) player: bool,
    pub(crate) interactive: bool,
    pub(crate) hit_points: Option<HitPoints>,
    pub(crate) remaining_turns: Option<u32>,
    pub(crate) solid: bool,
}

impl Default for PackedEntity {
    fn default() -> Self {
        Self {
            foreground_tile: None,
            light: None,
            npc: false,
            player: false,
            hit_points: None,
            interactive: false,
            remaining_turns: None,
            solid: false,
        }
    }
}

impl PackedEntity {
    pub(crate) fn spark() -> Self {
        let light = PackedLight::new(rgb24(0, 200, 200), 30, Rational::new(1, 10));
        Self {
            foreground_tile: Some(ForegroundTile::Spark),
            light: Some(light),
            npc: false,
            player: false,
            hit_points: None,
            interactive: false,
            ..Default::default()
        }
    }
    pub(crate) fn flame() -> Self {
        let light = PackedLight::new(rgb24(255, 120, 0), 30, Rational::new(1, 10));
        Self {
            foreground_tile: Some(ForegroundTile::Flame),
            light: Some(light),
            npc: false,
            player: false,
            hit_points: Some(HitPoints::new(3, 3)),
            interactive: true,
            ..Default::default()
        }
    }
    pub(crate) fn altar() -> Self {
        let light = PackedLight::new(rgb24(0, 200, 50), 30, Rational::new(1, 10));
        Self {
            foreground_tile: Some(ForegroundTile::Altar),
            light: Some(light),
            npc: false,
            player: false,
            hit_points: Some(HitPoints::new(1, 1)),
            interactive: true,
            ..Default::default()
        }
    }
    pub(crate) fn fountain() -> Self {
        let light = PackedLight::new(rgb24(50, 100, 200), 30, Rational::new(1, 10));
        Self {
            foreground_tile: Some(ForegroundTile::Fountain),
            light: Some(light),
            npc: false,
            player: false,
            hit_points: Some(HitPoints::new(1, 1)),
            interactive: true,
            ..Default::default()
        }
    }

    pub(crate) fn blink() -> Self {
        let light = PackedLight::new(rgb24(0, 255, 255), 30, Rational::new(1, 10));
        Self {
            foreground_tile: Some(ForegroundTile::Blink0),
            light: Some(light),
            npc: false,
            player: false,
            hit_points: None,
            interactive: false,
            ..Default::default()
        }
    }
    pub(crate) fn glow(colour: Rgb24) -> Self {
        let light = PackedLight::new(colour, 30, Rational::new(1, 10));
        Self {
            foreground_tile: None,
            light: Some(light),
            npc: false,
            player: false,
            hit_points: None,
            interactive: false,
            ..Default::default()
        }
    }
    pub(crate) fn block() -> Self {
        Self {
            foreground_tile: Some(ForegroundTile::Block),
            remaining_turns: Some(4),
            solid: true,
            ..Default::default()
        }
    }
    pub(crate) fn player() -> Self {
        let player_light = PackedLight::new(grey24(128), 30, Rational::new(1, 10));
        Self {
            foreground_tile: Some(ForegroundTile::Player),
            light: Some(player_light),
            npc: false,
            player: true,
            hit_points: Some(HitPoints::new(2, 4)),
            interactive: false,
            ..Default::default()
        }
    }
    pub(crate) fn bumper() -> Self {
        Self {
            foreground_tile: Some(ForegroundTile::Bumper),
            light: None,
            npc: true,
            player: false,
            hit_points: Some(HitPoints::new(2, 3)),
            interactive: false,
            ..Default::default()
        }
    }
    pub(crate) fn caster() -> Self {
        Self {
            foreground_tile: Some(ForegroundTile::Caster),
            light: None,
            npc: true,
            player: false,
            hit_points: Some(HitPoints::new(1, 2)),
            interactive: false,
            ..Default::default()
        }
    }
    pub(crate) fn healer() -> Self {
        Self {
            foreground_tile: Some(ForegroundTile::Healer),
            light: None,
            npc: true,
            player: false,
            hit_points: Some(HitPoints::new(1, 1)),
            interactive: false,
            ..Default::default()
        }
    }
}

pub enum ProjectileMove {
    Continue,
    HitObstacle,
    HitCharacter(EntityId),
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
    interactive_count: usize,
    solid_count: usize,
}

impl WorldCell {
    fn new(background_tile: BackgroundTile) -> Self {
        Self {
            background_tile,
            entities: HashSet::new(),
            npc_count: 0,
            player_count: 0,
            interactive_count: 0,
            solid_count: 0,
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
            || self.interactive_count != 0
            || self.solid_count != 0
    }
    pub fn contains_npc(&self) -> bool {
        self.npc_count > 0
    }
    pub fn contains_player(&self) -> bool {
        self.player_count > 0
    }
    pub fn is_interactive(&self) -> bool {
        self.interactive_count > 0
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
    lights: HashMap<LightId, Light>,
    entities: HashMap<EntityId, Entity>,
    next_id: EntityId,
    next_light_id: LightId,
    npc_ids: HashSet<EntityId>,
    remove_in_turns: HashMap<EntityId, u32>,
}

#[derive(Debug)]
pub enum CancelAction {
    MoveIntoSolidCell,
    MoveIntoNpc,
    OutOfBounds,
    OutOfRange,
    DestinationNotVisible,
    LocationBlocked,
    NoEntity,
    NoField,
    NothingToAttack,
    AlreadyFullHitPoints,
    InvalidCard,
    NotEnoughEnergy,
}

pub(crate) enum ApplyAction {
    Done,
    Animation(Animation),
    Interact(EntityId),
}

fn move_entity_to_coord(
    coord: Coord,
    entity: &mut Entity,
    grid: &mut Grid<WorldCell>,
    lights: &mut HashMap<LightId, Light>,
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
        let light = lights.get_mut(&light_index).unwrap();
        light.coord = entity.coord;
    }
}

impl World {
    pub(crate) fn set_heal_countdown(
        &mut self,
        id: EntityId,
        heal_countdown: Option<u32>,
    ) -> Result<ApplyAction, CancelAction> {
        self.entities.get_mut(&id).unwrap().heal_countdown = heal_countdown;
        Ok(ApplyAction::Done)
    }
    pub(crate) fn new(size: Size) -> Self {
        Self {
            grid: Grid::new_default(size),
            lights: HashMap::new(),
            entities: HashMap::new(),
            next_id: 0,
            next_light_id: 0,
            npc_ids: HashSet::new(),
            remove_in_turns: HashMap::new(),
        }
    }
    pub(crate) fn pack_entity(&self, id: EntityId) -> PackedEntity {
        let entity = self.entities.get(&id).unwrap();
        PackedEntity {
            foreground_tile: entity.foreground_tile,
            light: entity
                .light_index
                .map(|index| self.lights.get(&index).unwrap().pack()),
            npc: entity.npc,
            player: entity.player,
            hit_points: entity.hit_points,
            interactive: entity.interactive,
            remaining_turns: self.remove_in_turns.get(&id).cloned(),
            solid: entity.solid,
        }
    }
    pub(crate) fn lights(&self) -> &HashMap<LightId, Light> {
        &self.lights
    }
    pub fn grid(&self) -> &Grid<WorldCell> {
        &self.grid
    }
    pub fn entities(&self) -> &Entities {
        &self.entities
    }
    pub(crate) fn set_foreground(&mut self, id: EntityId, foreground: ForegroundTile) {
        if let Some(entity) = self.entities.get_mut(&id) {
            entity.foreground_tile = Some(foreground);
        }
    }
    pub(crate) fn set_light_params(
        &mut self,
        id: EntityId,
        colour: Rgb24,
        diminish: Rational,
    ) {
        if let Some(entity) = self.entities.get_mut(&id) {
            if let Some(light_index) = entity.light_index {
                if let Some(light) = self.lights.get_mut(&light_index) {
                    light.colour = colour;
                    light.diminish = diminish;
                }
            }
        }
    }
    pub(crate) fn reduce_remaining_turns(&mut self) {
        let mut ids_to_remove = Vec::new();
        for (&id, count) in self.remove_in_turns.iter_mut() {
            if *count == 0 {
                ids_to_remove.push(id);
            } else {
                *count -= 1;
            }
        }
        for id in ids_to_remove {
            self.remove_entity(id);
        }
    }
    pub(crate) fn add_entity(&mut self, coord: Coord, entity: PackedEntity) -> EntityId {
        let PackedEntity {
            foreground_tile,
            light,
            npc,
            player,
            hit_points,
            interactive,
            remaining_turns,
            solid,
        } = entity;
        let id = self.next_id;
        self.next_id += 1;
        let light_index = light.map(|packed_light| {
            let light_index = self.next_light_id;
            self.next_light_id += 1;
            self.lights.insert(light_index, packed_light.light(coord));
            light_index
        });
        if let Some(remaining_turns) = remaining_turns {
            self.remove_in_turns.insert(id, remaining_turns);
        }
        let entity = Entity {
            id,
            coord,
            foreground_tile,
            light_index,
            npc,
            player,
            taking_damage_in_direction: None,
            hit_points,
            interactive,
            heal_countdown: None,
            solid,
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
            if interactive {
                cell.interactive_count += 1;
            }
            if solid {
                cell.solid_count += 1;
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

    const BLINK_RANGE: u32 = 8;

    pub(crate) fn spark_in_direction(
        &mut self,
        id: EntityId,
        direction: CardinalDirection,
    ) -> Result<ApplyAction, CancelAction> {
        if let Some(entity) = self.entities.get(&id) {
            let id = self
                .add_entity(entity.coord() + direction.coord(), PackedEntity::spark());
            Ok(ApplyAction::Animation(Animation::spark(id, direction)))
        } else {
            Err(CancelAction::NoEntity)
        }
    }

    pub(crate) fn blink_entity_to_coord(
        &mut self,
        id: EntityId,
        coord: Coord,
    ) -> Result<ApplyAction, CancelAction> {
        if let Some(entity) = self.entities.get_mut(&id) {
            if let Some(cell) = self.grid.get(coord) {
                if cell.is_solid() {
                    Err(CancelAction::MoveIntoSolidCell)
                } else if cell.contains_npc() {
                    Err(CancelAction::MoveIntoNpc)
                } else {
                    let original_coord = entity.coord;
                    if original_coord.manhattan_distance(coord) <= Self::BLINK_RANGE {
                        move_entity_to_coord(
                            coord,
                            entity,
                            &mut self.grid,
                            &mut self.lights,
                        );
                        Ok(ApplyAction::Animation(Animation::blink(original_coord)))
                    } else {
                        Err(CancelAction::OutOfRange)
                    }
                }
            } else {
                Err(CancelAction::OutOfBounds)
            }
        } else {
            Err(CancelAction::NoEntity)
        }
    }

    pub(crate) fn heal(
        &mut self,
        id: EntityId,
        by: u32,
    ) -> Result<ApplyAction, CancelAction> {
        if let Some(entity) = self.entities.get_mut(&id) {
            if let Some(hit_points) = entity.hit_points.as_mut() {
                if hit_points.current < hit_points.max {
                    hit_points.current = (hit_points.current + by).min(hit_points.max);
                    Ok(ApplyAction::Done)
                } else {
                    Err(CancelAction::AlreadyFullHitPoints)
                }
            } else {
                Err(CancelAction::NoField)
            }
        } else {
            Err(CancelAction::NoEntity)
        }
    }

    pub(crate) fn bump_npc_in_direction(
        &mut self,
        id: EntityId,
        direction: CardinalDirection,
    ) -> Result<ApplyAction, CancelAction> {
        if let Some(entity) = self.entities.get_mut(&id) {
            let coord = entity.coord + direction.coord();
            if let Some(cell) = self.grid.get(coord) {
                if entity.player && cell.contains_npc() {
                    let id = cell
                        .entity_iter(&self.entities)
                        .find_map(|e| if e.npc { Some(e.id) } else { None })
                        .unwrap();
                    Ok(ApplyAction::Animation(Animation::damage(id, direction)))
                } else {
                    Err(CancelAction::NothingToAttack)
                }
            } else {
                Err(CancelAction::OutOfBounds)
            }
        } else {
            Err(CancelAction::NoEntity)
        }
    }

    pub(crate) fn can_move_projectile_in_direction(
        &self,
        id: EntityId,
        direction: CardinalDirection,
    ) -> Result<ProjectileMove, CancelAction> {
        if let Some(entity) = self.entities.get(&id) {
            let coord = entity.coord + direction.coord();
            if let Some(cell) = self.grid.get(coord) {
                if cell.contains_player() || cell.contains_npc() {
                    let character = cell
                        .entity_iter(&self.entities)
                        .find_map(|e| if e.player || e.npc { Some(e.id) } else { None })
                        .unwrap();
                    Ok(ProjectileMove::HitCharacter(character))
                } else if cell.is_solid() {
                    Ok(ProjectileMove::HitObstacle)
                } else {
                    Ok(ProjectileMove::Continue)
                }
            } else {
                Err(CancelAction::OutOfBounds)
            }
        } else {
            Err(CancelAction::NoEntity)
        }
    }

    pub(crate) fn move_entity_in_direction(
        &mut self,
        id: EntityId,
        direction: CardinalDirection,
    ) {
        if let Some(entity) = self.entities.get_mut(&id) {
            let coord = entity.coord + direction.coord();
            move_entity_to_coord(coord, entity, &mut self.grid, &mut self.lights);
        }
    }
    pub(crate) fn move_entity_in_direction_with_attack_policy(
        &mut self,
        id: EntityId,
        direction: CardinalDirection,
    ) -> Result<ApplyAction, CancelAction> {
        if let Some(entity) = self.entities.get_mut(&id) {
            let coord = entity.coord + direction.coord();
            if let Some(cell) = self.grid.get(coord) {
                if entity.player && cell.is_interactive() {
                    let interactive_id = cell
                        .entity_iter(self.entities())
                        .find(|e| e.interactive)
                        .map(|e| e.id)
                        .unwrap();
                    Ok(ApplyAction::Interact(interactive_id))
                } else if cell.is_solid() {
                    Err(CancelAction::MoveIntoSolidCell)
                } else if cell.contains_npc() {
                    Err(CancelAction::MoveIntoNpc)
                } else if entity.npc && cell.contains_player() {
                    let id = cell
                        .entity_iter(&self.entities)
                        .find_map(|e| if e.player { Some(e.id) } else { None })
                        .unwrap();
                    Ok(ApplyAction::Animation(Animation::damage(id, direction)))
                } else {
                    move_entity_to_coord(coord, entity, &mut self.grid, &mut self.lights);
                    Ok(ApplyAction::Done)
                }
            } else {
                Err(CancelAction::OutOfBounds)
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
                        ForegroundTile::Blink0 | ForegroundTile::Blink1 => 0,
                        ForegroundTile::Spark => 0,
                        ForegroundTile::Block => 255,
                        ForegroundTile::Caster => 0,
                        ForegroundTile::Healer => 0,
                        ForegroundTile::Player => 0,
                        ForegroundTile::Stairs => 0,
                        ForegroundTile::Flame => 0,
                        ForegroundTile::Altar => 0,
                        ForegroundTile::Fountain => 0,
                        ForegroundTile::Bumper => 0,
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
                hit_points.current = hit_points.current.saturating_sub(damage);
                if hit_points.current == 0 {
                    self.remove_entity(id);
                }
            }
        }
    }

    pub(crate) fn increase_max_hit_points(&mut self, id: EntityId, by: u32) {
        if let Some(entity) = self.entities.get_mut(&id) {
            if let Some(hit_points) = entity.hit_points.as_mut() {
                hit_points.max += by;
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
                if entity.interactive {
                    cell.interactive_count -= 1;
                }
                if entity.solid {
                    cell.solid_count -= 1;
                }
            }
            if let Some(light_index) = entity.light_index {
                self.lights.remove(&light_index);
            }
        }
    }
    pub(crate) fn set_light_diminish_denom(&mut self, id: EntityId, denom: u32) {
        if let Some(entity) = self.entities.get(&id) {
            if let Some(light_id) = entity.light_index {
                if let Some(light) = self.lights.get_mut(&light_id) {
                    light.diminish.denom = denom;
                }
            }
        }
    }
    pub(crate) fn increase_light_radius(&mut self, id: EntityId, by: u32) {
        if let Some(entity) = self.entities.get(&id) {
            if let Some(light_id) = entity.light_index {
                if let Some(light) = self.lights.get_mut(&light_id) {
                    let distance_squared = light.range.distance_squared();
                    light.range =
                        vision_distance::Circle::new_squared(distance_squared + by);
                }
            }
        }
    }
}
