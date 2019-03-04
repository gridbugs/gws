extern crate coord_2d;
extern crate direction;
extern crate grid_2d;
extern crate rand;
#[macro_use]
extern crate serde;
extern crate grid_search;
extern crate hashbrown;
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
pub struct Gws {
    world: World,
    visible_area: VisibileArea,
    pathfinding: PathfindingContext,
    player_id: EntityId,
    npc_ids: Vec<EntityId>,
}

pub struct ToRender<'a> {
    pub world: &'a World,
    pub visible_area: &'a VisibileArea,
    pub player: &'a Entity,
}

#[allow(dead_code)]
enum TerrainChoice {
    StringDemo,
    WfcIceCave(Size),
}

const TERRAIN_CHOICE: TerrainChoice = TerrainChoice::WfcIceCave(Size::new_u16(60, 40));

#[derive(Clone)]
pub struct BetweenLevels {
    player: PackedEntity,
}

pub enum Tick {
    ExitLevel(BetweenLevels),
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
        let pathfinding = PathfindingContext::new(size);
        let npc_ids = Vec::new();
        let mut s = Self {
            world,
            visible_area,
            player_id,
            pathfinding,
            npc_ids,
        };
        s.update_visible_area();
        s
    }

    pub fn tick<I: IntoIterator<Item = Input>, R: Rng>(
        &mut self,
        inputs: I,
        rng: &mut R,
    ) -> Option<Tick> {
        let _ = rng;
        self.npc_ids.clear();
        if let Some(input) = inputs.into_iter().next() {
            match input {
                Input::Move(direction) => {
                    let player_coord = self
                        .world
                        .move_entity_in_direction(self.player_id, direction);
                    self.pathfinding.update(player_coord, &self.world);
                }
            }
            for &id in self.world.npc_ids() {
                self.npc_ids.push(id);
            }
        }
        for id in self.npc_ids.drain(..) {
            let coord = self.world.entities().get(&id).unwrap().coord();
            if let Some(direction) = self
                .pathfinding
                .direction_towards_player(coord, &self.world)
            {
                self.world.move_entity_in_direction(id, direction);
            }
        }

        self.update_visible_area();
        if let Some(cell) = self.world.grid().get(self.player().coord()) {
            for entity in cell.entity_iter(self.world.entities()) {
                if entity.foreground_tile() == Some(ForegroundTile::Stairs) {
                    return Some(Tick::ExitLevel(BetweenLevels {
                        player: self.world.pack_entity(self.player_id),
                    }));
                }
            }
        }
        None
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
        }
    }
}
