extern crate coord_2d;
extern crate direction;
extern crate grid_2d;
extern crate rand;
#[macro_use]
extern crate serde;
extern crate grid_search;
extern crate hashbrown;
extern crate line_2d;
extern crate rgb24;
extern crate shadowcast;
extern crate wfc;

mod pathfinding;
mod terrain;
mod vision;
mod world;

use crate::pathfinding::*;
use crate::vision::*;
pub use crate::world::*;
use coord_2d::*;
use direction::*;
use rand::Rng;
use rgb24::*;
use std::time::Duration;

const NPC_VISION_RANGE: usize = 16;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Input {
    Move(CardinalDirection),
    PlayCard { slot: usize, param: CardParam },
    Wait,
    Interact(InteractiveParam),
}

pub mod input {
    use super::*;
    pub const UP: Input = Input::Move(CardinalDirection::North);
    pub const DOWN: Input = Input::Move(CardinalDirection::South);
    pub const LEFT: Input = Input::Move(CardinalDirection::West);
    pub const RIGHT: Input = Input::Move(CardinalDirection::East);
    pub const WAIT: Input = Input::Wait;
    pub fn play_card(slot: usize, param: CardParam) -> Input {
        Input::PlayCard { slot, param }
    }
    pub fn interact(param: InteractiveParam) -> Input {
        Input::Interact(param)
    }
}

const INITIAL_DRAW_COUNTDOWN: u32 = 12;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct DrawCountdown {
    pub current: u32,
    pub max: u32,
}

impl DrawCountdown {
    fn new() -> Self {
        Self {
            current: INITIAL_DRAW_COUNTDOWN,
            max: INITIAL_DRAW_COUNTDOWN,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Gws {
    world: World,
    visible_area: VisibileArea,
    pathfinding: PathfindingContext,
    player_id: EntityId,
    animation: Vec<Animation>,
    turn: Turn,
    hand: Vec<Option<Card>>,
    deck: Vec<Card>,
    spent: Vec<Card>,
    waste: Vec<Card>,
    draw_countdown: DrawCountdown,
}

pub struct ToRender<'a> {
    pub world: &'a World,
    pub visible_area: &'a VisibileArea,
    pub player: &'a Entity,
    pub commitment_grid: &'a CommitmentGrid,
}

#[allow(dead_code)]
enum TerrainChoice {
    StringDemo,
    WfcIceCave(Size),
}

//const TERRAIN_CHOICE: TerrainChoice = TerrainChoice::WfcIceCave(Size::new_u16(60, 40));
const TERRAIN_CHOICE: TerrainChoice = TerrainChoice::StringDemo;

#[derive(Clone)]
pub struct BetweenLevels {
    player: PackedEntity,
}

pub enum End {
    ExitLevel(BetweenLevels),
    PlayerDied,
}

#[derive(Debug, Clone, Copy)]
pub struct Interactive {
    pub entity_id: EntityId,
    pub typ: InteractiveType,
}

#[derive(Debug, Clone, Copy)]
pub enum InteractiveType {
    Flame,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum InteractiveParam {
    Flame { card: Card, entity_id: EntityId },
}

pub enum Tick {
    End(End),
    CancelAction(CancelAction),
    Interact(Interactive),
}

#[derive(Clone, Copy, Serialize, Deserialize)]
enum AnimationState {
    DamageStart {
        id: EntityId,
        direction: CardinalDirection,
    },
    DamageEnd {
        id: EntityId,
    },
    BlinkStart {
        coord: Coord,
    },
    Blink {
        id: EntityId,
        stage: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Card {
    Bump,
    Blink,
    Heal,
}

impl Card {
    pub fn cost(self) -> u32 {
        match self {
            Card::Blink => 3,
            Card::Bump => 2,
            Card::Heal => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CardParam {
    Coord(Coord),
    CardinalDirection(CardinalDirection),
    Confirm,
}

const DAMAGE_ANIMATION_PERIOD: Duration = Duration::from_millis(250);
const BLINK_ANIMATION_PERIOD: Duration = Duration::from_millis(50);

impl AnimationState {
    fn update(self, world: &mut World) -> Option<Animation> {
        match self {
            AnimationState::DamageStart { id, direction } => {
                world.set_taking_damage_in_direction(id, Some(direction));
                Some(Animation::new(
                    DAMAGE_ANIMATION_PERIOD,
                    AnimationState::DamageEnd { id },
                ))
            }
            AnimationState::DamageEnd { id } => {
                world.set_taking_damage_in_direction(id, None);
                world.deal_damage(id, 1);
                None
            }
            AnimationState::BlinkStart { coord } => {
                let id = world.add_entity(coord, PackedEntity::blink());
                Some(Animation::new(
                    BLINK_ANIMATION_PERIOD,
                    AnimationState::Blink { id, stage: 0 },
                ))
            }
            AnimationState::Blink { id, stage } => {
                if stage == 0 {
                    world.set_foreground(id, ForegroundTile::Blink1);
                    world.set_light_params(id, rgb24(0, 128, 128), Rational::new(1, 5));
                    Some(Animation::new(
                        BLINK_ANIMATION_PERIOD,
                        AnimationState::Blink { id, stage: 1 },
                    ))
                } else {
                    world.remove_entity(id);
                    None
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum Turn {
    Player,
    Engine,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
struct Animation {
    next_update_in: Duration,
    state: AnimationState,
}

impl Animation {
    pub fn new(next_update_in: Duration, state: AnimationState) -> Self {
        Self {
            next_update_in,
            state,
        }
    }
    fn tick(self, period: Duration, world: &mut World) -> Option<Self> {
        let Animation {
            next_update_in,
            state,
        } = self;
        if period >= next_update_in {
            state.update(world)
        } else {
            Some(Self {
                next_update_in: next_update_in - period,
                state,
            })
        }
    }
    pub fn damage(id: EntityId, direction: CardinalDirection) -> Self {
        Self::new(
            Duration::from_secs(0),
            AnimationState::DamageStart { id, direction },
        )
    }
    pub fn blink(coord: Coord) -> Self {
        Self::new(Duration::from_secs(0), AnimationState::BlinkStart { coord })
    }
}

impl Gws {
    pub fn new<R: Rng>(
        between_levels: Option<BetweenLevels>,
        rng: &mut R,
        debug_terrain_string: Option<&str>,
    ) -> Self {
        let terrain::TerrainDescription {
            size,
            player_coord,
            instructions,
        } = match TERRAIN_CHOICE {
            TerrainChoice::StringDemo => terrain::from_str(
                debug_terrain_string.unwrap_or(include_str!("terrain_string.txt")),
            ),
            TerrainChoice::WfcIceCave(size) => terrain::wfc_ice_cave(size, rng),
        };
        let player = match between_levels {
            None => PackedEntity::player(),
            Some(BetweenLevels { player }) => player,
        };
        let mut world = World::new(size);
        for instruction in instructions {
            world.interpret_instruction(instruction);
        }
        let player_id = world.add_entity(player_coord, player);
        let visible_area = VisibileArea::new(size);
        let mut pathfinding = PathfindingContext::new(size);
        pathfinding.update_player_coord(player_coord, &world);
        for &id in world.npc_ids() {
            pathfinding.commit_to_moving_towards_player(id, &world);
        }
        let mut s = Self {
            world,
            visible_area,
            player_id,
            pathfinding,
            animation: Vec::new(),
            turn: Turn::Player,
            hand: vec![
                Some(Card::Blink),
                Some(Card::Heal),
                Some(Card::Bump),
                Some(Card::Bump),
                Some(Card::Blink),
                Some(Card::Heal),
                Some(Card::Bump),
            ],
            draw_countdown: DrawCountdown::new(),
            deck: vec![
                Card::Bump,
                Card::Bump,
                Card::Bump,
                Card::Bump,
                Card::Bump,
                Card::Bump,
                Card::Heal,
                Card::Heal,
                Card::Heal,
                Card::Heal,
                Card::Heal,
                Card::Blink,
                Card::Blink,
                Card::Blink,
                Card::Blink,
            ],
            spent: Vec::new(),
            waste: Vec::new(),
        };
        s.update_visible_area();
        s
    }

    pub fn draw_countdown(&self) -> &DrawCountdown {
        &self.draw_countdown
    }

    fn player_turn(&mut self, input: Input) -> Result<ApplyAction, CancelAction> {
        let (result, cost) = match input {
            Input::Interact(param) => match param {
                InteractiveParam::Flame { card, entity_id } => {
                    let index = self
                        .spent
                        .iter()
                        .position(|&c| c == card)
                        .expect("no such card in spent");
                    self.spent.swap_remove(index);
                    self.world.deal_damage(self.player_id, 1);
                    self.world.deal_damage(entity_id, 1);
                    (Ok(ApplyAction::Done), 0)
                }
            },
            Input::Move(direction) => {
                let result = self.world.move_entity_in_direction_with_attack_policy(
                    self.player_id,
                    direction,
                );
                (result, 1)
            }
            Input::Wait => (Ok(ApplyAction::Done), 1),
            Input::PlayCard { slot, param } => {
                let card = if let Some(&card) = self.hand.get(slot) {
                    card
                } else {
                    return Err(CancelAction::InvalidCard);
                };
                let card = if let Some(card) = card {
                    card
                } else {
                    return Err(CancelAction::InvalidCard);
                };
                if card.cost() > self.draw_countdown.current {
                    return Err(CancelAction::NotEnoughEnergy);
                }
                let result = match (card, param) {
                    (Card::Blink, CardParam::Coord(coord)) => self.blink(coord),
                    (Card::Bump, CardParam::CardinalDirection(direction)) => {
                        self.bump(direction)
                    }
                    (Card::Heal, CardParam::Confirm) => self.heal(1),
                    _ => return Err(CancelAction::InvalidCard),
                };
                if result.is_ok() {
                    self.hand[slot] = None;
                    self.spent.push(card);
                }
                (result, card.cost())
            }
        };
        // TODO this is messy
        match result {
            Err(_) | Ok(ApplyAction::Interact(_)) => (),
            _ => {
                let should_draw = if let Some(current) =
                    self.draw_countdown.current.checked_sub(cost)
                {
                    current == 0
                } else {
                    true
                };
                if should_draw {
                    self.draw_hand();
                    self.draw_countdown.current = self.draw_countdown.max;
                } else {
                    self.draw_countdown.current -= cost;
                }
            }
        }
        result
    }

    fn draw_hand(&mut self) {
        for slot in self.hand.iter_mut() {
            if let Some(card) = *slot {
                self.waste.push(card);
            }
            *slot = self.deck.pop();
        }
    }

    fn blink(&mut self, coord: Coord) -> Result<ApplyAction, CancelAction> {
        if self.visible_area.is_visible(coord)
            && self.visible_area.light_colour(coord) != grey24(0)
        {
            self.world.blink_entity_to_coord(self.player_id, coord)
        } else {
            Err(CancelAction::DestinationNotVisible)
        }
    }

    fn bump(
        &mut self,
        direction: CardinalDirection,
    ) -> Result<ApplyAction, CancelAction> {
        self.world.bump_npc_in_direction(self.player_id, direction)
    }

    fn heal(&mut self, by: u32) -> Result<ApplyAction, CancelAction> {
        self.world.heal(self.player_id, by)
    }

    fn engine_turn(&mut self) {
        for &(id, direction) in self.pathfinding.committed_movements().iter() {
            match self
                .world
                .move_entity_in_direction_with_attack_policy(id, direction)
            {
                Ok(ApplyAction::Done) => (),
                Ok(ApplyAction::Animation(animation)) => self.animation.push(animation),
                Ok(ApplyAction::Interact(_)) => (),
                Err(_) => (),
            }
        }
        let player_coord = self.player().coord();
        self.pathfinding
            .update_player_coord(player_coord, &self.world);
        for &id in self.world.npc_ids() {
            let npc_coord = self.world.entities().get(&id).unwrap().coord();
            if self
                .world
                .can_see(npc_coord, player_coord, NPC_VISION_RANGE)
            {
                self.pathfinding
                    .commit_to_moving_towards_player(id, &self.world);
            }
        }
    }

    fn check_end(&self) -> Option<End> {
        let player = self.player();
        if let Some(cell) = self.world.grid().get(player.coord()) {
            for entity in cell.entity_iter(self.world.entities()) {
                if entity.foreground_tile() == Some(ForegroundTile::Stairs) {
                    return Some(End::ExitLevel(BetweenLevels {
                        player: self.world.pack_entity(self.player_id),
                    }));
                }
            }
        }
        if player.hit_points().unwrap().current == 0 {
            return Some(End::PlayerDied);
        }
        None
    }

    pub fn animate(&mut self, period: Duration) {
        if let Some(animation) = self.animation.pop() {
            if let Some(animation) = animation.tick(period, &mut self.world) {
                self.animation.push(animation);
            }
        }
    }

    pub fn tick<I: IntoIterator<Item = Input>, R: Rng>(
        &mut self,
        inputs: I,
        period: Duration,
        rng: &mut R,
    ) -> Option<Tick> {
        let _ = rng;
        self.animate(period);
        if self.animation.is_empty() {
            if self.turn == Turn::Player {
                if let Some(input) = inputs.into_iter().next() {
                    match self.player_turn(input) {
                        Err(cancel) => {
                            return Some(Tick::CancelAction(cancel));
                        }
                        Ok(ApplyAction::Done) => self.turn = Turn::Engine,
                        Ok(ApplyAction::Interact(entity_id)) => {
                            let entity = self.world.entities().get(&entity_id).unwrap();
                            let typ = match entity.foreground_tile().unwrap() {
                                ForegroundTile::Flame => InteractiveType::Flame,
                                _ => panic!("illegal interactive"),
                            };
                            return Some(Tick::Interact(Interactive { typ, entity_id }));
                        }
                        Ok(ApplyAction::Animation(animation)) => {
                            self.turn = Turn::Engine;
                            self.animation.push(animation);
                        }
                    }
                };
            }
        }
        if self.animation.is_empty() {
            if self.turn == Turn::Engine {
                self.engine_turn();
                self.turn = Turn::Player;
            }
        }
        self.animate(Duration::from_secs(0));
        self.update_visible_area();
        self.check_end().map(Tick::End)
    }

    fn player(&self) -> &Entity {
        self.world.entities().get(&self.player_id).unwrap()
    }

    fn update_visible_area(&mut self) {
        self.visible_area.update(self.player().coord(), &self.world);
    }

    pub fn to_render(&self) -> ToRender {
        ToRender {
            world: &self.world,
            visible_area: &self.visible_area,
            player: self.player(),
            commitment_grid: self.pathfinding.commitment_grid(),
        }
    }

    pub fn hand(&self) -> &[Option<Card>] {
        &self.hand
    }

    pub fn deck(&self) -> &[Card] {
        &self.deck
    }

    pub fn spent(&self) -> &[Card] {
        &self.spent
    }

    pub fn waste(&self) -> &[Card] {
        &self.waste
    }
}
