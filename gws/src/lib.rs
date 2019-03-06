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
}

pub mod input {
    use super::*;
    pub const UP: Input = Input::Move(CardinalDirection::North);
    pub const DOWN: Input = Input::Move(CardinalDirection::South);
    pub const LEFT: Input = Input::Move(CardinalDirection::West);
    pub const RIGHT: Input = Input::Move(CardinalDirection::East);
    pub fn play_card(slot: usize, param: CardParam) -> Input {
        Input::PlayCard { slot, param }
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

pub enum Tick {
    End(End),
    CancelAction(CancelAction),
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

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Card {
    Bump,
    Blink,
    Heal,
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

enum PlayerTurn {
    Done,
    Cancelled(CancelAction),
    Animation(Animation),
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
                None,
                Some(Card::Bump),
                Some(Card::Bump),
                Some(Card::Blink),
                Some(Card::Heal),
                None,
            ],
        };
        s.update_visible_area();
        s
    }

    fn handle_player_action_result(
        result: Result<ApplyAction, CancelAction>,
    ) -> PlayerTurn {
        match result {
            Ok(ApplyAction::Done) => PlayerTurn::Done,
            Ok(ApplyAction::Animation(animation)) => PlayerTurn::Animation(animation),
            Err(cancel) => PlayerTurn::Cancelled(cancel),
        }
    }

    fn player_turn(&mut self, input: Input) -> PlayerTurn {
        match input {
            Input::Move(direction) => Self::handle_player_action_result(
                self.world.move_entity_in_direction_with_attack_policy(
                    self.player_id,
                    direction,
                ),
            ),
            Input::PlayCard { slot, param } => {
                let card = if let Some(&card) = self.hand.get(slot) {
                    card
                } else {
                    return PlayerTurn::Cancelled(CancelAction::InvalidCard);
                };
                let card = if let Some(card) = card {
                    card
                } else {
                    return PlayerTurn::Cancelled(CancelAction::InvalidCard);
                };
                match (card, param) {
                    (Card::Blink, CardParam::Coord(coord)) => self.blink(coord),
                    (Card::Bump, CardParam::CardinalDirection(direction)) => {
                        self.bump(direction)
                    }
                    (Card::Heal, CardParam::Confirm) => self.heal(1),
                    _ => PlayerTurn::Cancelled(CancelAction::InvalidCard),
                }
            }
        }
    }

    fn blink(&mut self, coord: Coord) -> PlayerTurn {
        if self.visible_area.is_visible(coord) {
            Self::handle_player_action_result(
                self.world.blink_entity_to_coord(self.player_id, coord),
            )
        } else {
            PlayerTurn::Cancelled(CancelAction::DestinationNotVisible)
        }
    }

    fn bump(&mut self, direction: CardinalDirection) -> PlayerTurn {
        Self::handle_player_action_result(
            self.world.bump_npc_in_direction(self.player_id, direction),
        )
    }

    fn heal(&mut self, by: u32) -> PlayerTurn {
        Self::handle_player_action_result(self.world.heal(self.player_id, by))
    }

    fn engine_turn(&mut self) {
        for &(id, direction) in self.pathfinding.committed_movements().iter() {
            match self
                .world
                .move_entity_in_direction_with_attack_policy(id, direction)
            {
                Ok(ApplyAction::Done) => (),
                Ok(ApplyAction::Animation(animation)) => self.animation.push(animation),
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
                let player_turn = if let Some(input) = inputs.into_iter().next() {
                    self.player_turn(input)
                } else {
                    PlayerTurn::Cancelled(CancelAction::NoInput)
                };
                match player_turn {
                    PlayerTurn::Cancelled(cancel) => {
                        return Some(Tick::CancelAction(cancel));
                    }
                    PlayerTurn::Done => self.turn = Turn::Engine,
                    PlayerTurn::Animation(animation) => {
                        self.turn = Turn::Engine;
                        self.animation.push(animation);
                    }
                }
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
}
