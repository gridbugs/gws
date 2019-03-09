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

pub use crate::pathfinding::*;
use crate::vision::*;
pub use crate::world::*;
use coord_2d::*;
use direction::*;
use line_2d::*;
use rand::seq::SliceRandom;
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

const INITIAL_DRAW_COUNTDOWN: u32 = 40;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct DrawCountdown {
    pub current: u32,
    pub max: u32,
}

const MAX_NUM_CARDS: usize = 8;

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
    burnt: Vec<Card>,
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

const TERRAIN_CHOICE: TerrainChoice = TerrainChoice::WfcIceCave(Size::new_u16(60, 40));
//const TERRAIN_CHOICE: TerrainChoice = TerrainChoice::StringDemo;

#[derive(Clone)]
pub struct BetweenLevels {
    player: PackedEntity,
    deck: Vec<Card>,
    burnt: Vec<Card>,
    hand_size: usize,
    max_draw_countdown: u32,
}

impl BetweenLevels {
    fn initial() -> Self {
        let player = PackedEntity::player();
        let deck = vec![
            Card::Spark,
            Card::Spark,
            Card::Spark,
            Card::Spark,
            Card::Spark,
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
        ];
        let burnt = Vec::new();
        let hand_size = 5;
        let max_draw_countdown = INITIAL_DRAW_COUNTDOWN;
        Self {
            player,
            deck,
            burnt,
            hand_size,
            max_draw_countdown,
        }
    }
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
    Altar,
    Fountain,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CharacterUpgrade {
    Life,
    Power,
    Hand,
    Vision,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum InteractiveParam {
    Flame {
        card: Card,
        entity_id: EntityId,
    },
    Altar {
        character_upgrade: CharacterUpgrade,
        card: Card,
        entity_id: EntityId,
    },
    Fountain {
        card: Card,
        entity_id: EntityId,
        count: usize,
    },
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
    Projectile {
        id: EntityId,
        direction: CardinalDirection,
        remaining_range: u32,
    },
    GlowFadeIn {
        id: EntityId,
        remaining_frames: u32,
        total_frames: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Card {
    Bump,
    Blink,
    Heal,
    Spark,
    Clog,
    Parasite,
    Drain,
}

const NEGATIVE_CARDS: &'static [Card] = &[Card::Clog, Card::Parasite, Card::Drain];
const POSITIVE_CARDS: &'static [Card] =
    &[Card::Bump, Card::Blink, Card::Heal, Card::Spark];

impl Card {
    pub fn cost(self) -> u32 {
        match self {
            Card::Blink => 20,
            Card::Bump => 10,
            Card::Heal => 5,
            Card::Spark => 20,
            Card::Clog => 10,
            Card::Parasite => 10,
            Card::Drain => 40,
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
const PROJECTILE_ANIMATION_PERIOD: Duration = Duration::from_millis(50);
const GLOW_FADE_IN_PERIOD: Duration = Duration::from_millis(50);

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
            AnimationState::Projectile {
                id,
                direction,
                remaining_range,
            } => {
                let next = match world.can_move_projectile_in_direction(id, direction) {
                    Err(_) => None,
                    Ok(ProjectileMove::HitObstacle) => None,
                    Ok(ProjectileMove::HitCharacter(id)) => {
                        world.deal_damage(id, 1);
                        None
                    }
                    Ok(ProjectileMove::Continue) => {
                        if remaining_range == 0 {
                            None
                        } else {
                            Some(Animation::new(
                                PROJECTILE_ANIMATION_PERIOD,
                                AnimationState::Projectile {
                                    remaining_range: remaining_range - 1,
                                    direction,
                                    id,
                                },
                            ))
                        }
                    }
                };
                if next.is_none() {
                    world.remove_entity(id);
                } else {
                    world.move_entity_in_direction(id, direction);
                }
                next
            }
            AnimationState::GlowFadeIn {
                id,
                remaining_frames,
                total_frames,
            } => {
                if remaining_frames == 0 {
                    world.remove_entity(id);
                    None
                } else {
                    world.set_light_diminish_denom(
                        id,
                        total_frames - remaining_frames + 1,
                    );
                    Some(Animation::new(
                        GLOW_FADE_IN_PERIOD,
                        AnimationState::GlowFadeIn {
                            remaining_frames: remaining_frames - 1,
                            total_frames,
                            id,
                        },
                    ))
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

const PROJECTILE_RANGE: u32 = 12;

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
    pub fn spark(id: EntityId, direction: CardinalDirection) -> Self {
        Self::new(
            Duration::from_secs(0),
            AnimationState::Projectile {
                id,
                direction,
                remaining_range: PROJECTILE_RANGE,
            },
        )
    }
    pub fn glow_fade_out(id: EntityId, remaining_frames: u32) -> Self {
        Self::new(
            Duration::from_secs(0),
            AnimationState::GlowFadeIn {
                id,
                remaining_frames,
                total_frames: remaining_frames,
            },
        )
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
        let BetweenLevels {
            player,
            mut deck,
            burnt,
            hand_size,
            max_draw_countdown,
        } = between_levels.unwrap_or_else(BetweenLevels::initial);
        deck.shuffle(rng);
        let draw_countdown = DrawCountdown {
            max: max_draw_countdown,
            current: max_draw_countdown,
        };
        let mut world = World::new(size);
        let hand = (0..hand_size).map(|_| None).collect::<Vec<_>>();
        for instruction in instructions {
            world.interpret_instruction(instruction);
        }
        let player_id = world.add_entity(player_coord, player);
        let visible_area = VisibileArea::new(size);
        let pathfinding = PathfindingContext::new(size);
        let mut s = Self {
            world,
            visible_area,
            player_id,
            pathfinding,
            animation: Vec::new(),
            turn: Turn::Player,
            hand,
            draw_countdown,
            deck,
            spent: Vec::new(),
            waste: Vec::new(),
            burnt,
        };
        s.engine_commit();
        s.draw_hand();
        s.update_visible_area();
        s
    }

    pub fn draw_countdown(&self) -> &DrawCountdown {
        &self.draw_countdown
    }

    fn player_turn<R: Rng>(
        &mut self,
        input: Input,
        rng: &mut R,
    ) -> Result<ApplyAction, CancelAction> {
        let (result, cost) = match input {
            Input::Interact(param) => match param {
                InteractiveParam::Flame { card, entity_id } => {
                    let index = self
                        .spent
                        .iter()
                        .position(|&c| c == card)
                        .expect("no such card in spent");
                    self.spent.swap_remove(index);
                    self.burnt.push(card);
                    self.world.deal_damage(self.player_id, 1);
                    self.world.deal_damage(entity_id, 1);
                    (Ok(ApplyAction::Done), 0)
                }
                InteractiveParam::Fountain {
                    card,
                    entity_id,
                    count,
                } => {
                    for _ in 0..count {
                        self.deck.push(card);
                    }
                    self.deck.shuffle(rng);
                    self.world.deal_damage(entity_id, 1);
                    (Ok(ApplyAction::Done), 0)
                }
                InteractiveParam::Altar {
                    character_upgrade,
                    entity_id,
                    card,
                } => {
                    use CharacterUpgrade::*;
                    match character_upgrade {
                        Life => self.world.increase_max_hit_points(self.player_id, 2),
                        Power => self.draw_countdown.max += 2,
                        Hand => self.hand.push(None),
                        Vision => self.world.increase_light_radius(self.player_id, 30),
                    }
                    self.deck.push(card);
                    self.deck.shuffle(rng);
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
                enum Deck {
                    Spent,
                    Burnt,
                }
                use Deck::*;
                let (result, deck) = match (card, param) {
                    (Card::Blink, CardParam::Coord(coord)) => (self.blink(coord), Spent),
                    (Card::Bump, CardParam::CardinalDirection(direction)) => {
                        (self.bump(direction), Spent)
                    }
                    (Card::Spark, CardParam::CardinalDirection(direction)) => {
                        (self.spark(direction), Spent)
                    }
                    (Card::Heal, CardParam::Confirm) => (self.heal(1), Spent),
                    (Card::Clog, CardParam::Confirm) => (Ok(ApplyAction::Done), Spent),
                    (Card::Parasite, CardParam::Confirm) => {
                        self.world.deal_damage(self.player_id, 2);
                        (Ok(ApplyAction::Done), Burnt)
                    }
                    (Card::Drain, CardParam::Confirm) => (Ok(ApplyAction::Done), Burnt),
                    _ => return Err(CancelAction::InvalidCard),
                };
                if result.is_ok() {
                    self.hand[slot] = None;
                    match deck {
                        Spent => self.spent.push(card),
                        Burnt => self.burnt.push(card),
                    }
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

    fn spark(
        &mut self,
        direction: CardinalDirection,
    ) -> Result<ApplyAction, CancelAction> {
        self.world.spark_in_direction(self.player_id, direction)
    }

    fn engine_turn(&mut self) {
        for &(id, direction, typ) in self.pathfinding.committed_actions().iter() {
            if let Some(entity) = self.world.entities().get(&id) {
                let result = match typ {
                    CommitmentType::Move => self
                        .world
                        .move_entity_in_direction_with_attack_policy(id, direction),
                    CommitmentType::Cast => self.world.spark_in_direction(id, direction),
                    CommitmentType::Heal(0) => {
                        let heal_range = 10;
                        let coord = entity.coord();
                        let to_heal = self
                            .world
                            .npc_ids()
                            .map(|id| self.world.entities().get(id).unwrap())
                            .filter_map(|e| {
                                if entity.coord().manhattan_distance(e.coord())
                                    < heal_range
                                {
                                    if let Some(hit_points) = e.hit_points() {
                                        if hit_points.current < hit_points.max {
                                            Some(e.id())
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();
                        for id in to_heal {
                            let _ = self.world.heal(id, 1);
                        }
                        let _ = self.world.set_heal_countdown(id, None);
                        Ok(ApplyAction::Animation(Animation::glow_fade_out(
                            self.world.add_entity(
                                coord,
                                PackedEntity::glow(rgb24(255, 255, 0)),
                            ),
                            10,
                        )))
                    }
                    CommitmentType::Heal(count) => {
                        self.world.set_heal_countdown(id, Some(count))
                    }
                };
                match result {
                    Ok(ApplyAction::Done) => (),
                    Ok(ApplyAction::Animation(animation)) => {
                        self.animation.push(animation)
                    }
                    Ok(ApplyAction::Interact(_)) => (),
                    Err(_) => (),
                }
            }
        }
        self.pathfinding.clear_commitments();
    }
    fn engine_commit(&mut self) {
        let player_coord = self.player().coord();
        self.pathfinding
            .update_player_coord(player_coord, &self.world);
        for &id in self.world.npc_ids() {
            let npc = self.world.entities().get(&id).unwrap();
            if self
                .world
                .can_see(npc.coord(), player_coord, NPC_VISION_RANGE)
            {
                if npc.foreground_tile() == Some(ForegroundTile::Caster)
                    && (npc.coord().x == player_coord.x
                        || npc.coord().y == player_coord.y)
                    && npc.coord().manhattan_distance(player_coord) < 8
                {
                    let mut clean_shot = true;
                    for coord in LineSegment::new(npc.coord(), player_coord)
                        .iter_config(Config::new().exclude_start().exclude_end())
                    {
                        if let Some(cell) = self.world.grid().get(coord) {
                            if cell.contains_npc() {
                                clean_shot = false;
                            } else if cell.is_solid() {
                                clean_shot = false;
                            }
                        }
                    }
                    if clean_shot {
                        self.pathfinding.commit_action(
                            id,
                            &self.world,
                            CommitmentType::Cast,
                        );
                    } else {
                        self.pathfinding.commit_action(
                            id,
                            &self.world,
                            CommitmentType::Move,
                        );
                    }
                } else if npc.foreground_tile() == Some(ForegroundTile::Healer) {
                    if let Some(heal_countdown) = npc.heal_countdown() {
                        if heal_countdown > 0 {
                            self.pathfinding.commit_action(
                                id,
                                &self.world,
                                CommitmentType::Heal(heal_countdown.saturating_sub(1)),
                            );
                        }
                    } else {
                        let heal_range = 8;
                        if let Some(_) = self
                            .world
                            .npc_ids()
                            .map(|id| self.world.entities().get(id).unwrap())
                            .find(|e| {
                                npc.coord().manhattan_distance(e.coord()) < heal_range
                                    && {
                                        if let Some(hit_points) = e.hit_points() {
                                            hit_points.current < hit_points.max
                                        } else {
                                            false
                                        }
                                    }
                            })
                        {
                            self.pathfinding.commit_action(
                                id,
                                &self.world,
                                CommitmentType::Heal(3),
                            );
                        } else {
                            self.pathfinding.commit_action(
                                id,
                                &self.world,
                                CommitmentType::Move,
                            );
                        }
                    }
                } else {
                    self.pathfinding
                        .commit_action(id, &self.world, CommitmentType::Move);
                }
            }
        }
        self.turn = Turn::Player;
    }

    fn between_levels(&self) -> BetweenLevels {
        let deck = self
            .deck()
            .iter()
            .chain(self.spent.iter())
            .chain(self.waste.iter())
            .chain(self.hand.iter().filter_map(|c| c.as_ref()))
            .cloned()
            .collect::<Vec<_>>();
        let burnt = self.burnt.clone();
        let player = self.world.pack_entity(self.player_id);
        BetweenLevels {
            deck,
            burnt,
            player,
            hand_size: self.hand.len(),
            max_draw_countdown: self.draw_countdown.max,
        }
    }

    fn check_end(&self) -> Option<End> {
        let player = self.player();
        if let Some(cell) = self.world.grid().get(player.coord()) {
            for entity in cell.entity_iter(self.world.entities()) {
                if entity.foreground_tile() == Some(ForegroundTile::Stairs) {
                    return Some(End::ExitLevel(self.between_levels()));
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
        if self.animation.is_empty() && self.turn == Turn::Engine {
            self.engine_commit();
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
                    match self.player_turn(input, rng) {
                        Err(cancel) => {
                            return Some(Tick::CancelAction(cancel));
                        }
                        Ok(ApplyAction::Done) => self.turn = Turn::Engine,
                        Ok(ApplyAction::Interact(entity_id)) => {
                            let entity = self.world.entities().get(&entity_id).unwrap();
                            let typ = match entity.foreground_tile().unwrap() {
                                ForegroundTile::Flame => InteractiveType::Flame,
                                ForegroundTile::Altar => InteractiveType::Altar,
                                ForegroundTile::Fountain => InteractiveType::Fountain,
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
    pub fn burnt(&self) -> &[Card] {
        &self.burnt
    }
    pub fn choose_upgrades<R: Rng>(
        &self,
        amount: usize,
        rng: &mut R,
    ) -> impl Iterator<Item = &'static CharacterUpgrade> {
        use CharacterUpgrade::*;
        const WITH_HAND: &'static [CharacterUpgrade] = &[Life, Power, Hand, Vision];
        const WITHOUT_HAND: &'static [CharacterUpgrade] = &[Life, Power, Vision];
        let slice = if self.hand.len() >= MAX_NUM_CARDS {
            WITHOUT_HAND
        } else {
            WITH_HAND
        };
        slice.choose_multiple(rng, amount)
    }
    pub fn choose_negative_cards<R: Rng>(
        &self,
        amount: usize,
        rng: &mut R,
    ) -> impl Iterator<Item = &'static Card> {
        NEGATIVE_CARDS.choose_multiple(rng, amount)
    }
    pub fn choose_positive_cards<R: Rng>(
        &self,
        amount: usize,
        rng: &mut R,
    ) -> impl Iterator<Item = &'static Card> {
        POSITIVE_CARDS.choose_multiple(rng, amount)
    }
}
