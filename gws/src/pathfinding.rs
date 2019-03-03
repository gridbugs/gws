use crate::world::*;
use direction::*;
use grid_search::*;

#[derive(Serialize, Deserialize)]
pub struct PathfindingContext {
    search: SearchContext<u32>,
    bfs: BfsContext,
    distance_to_player: UniformDistanceMap<u32, DirectionsCardinal>,
    path: Vec<Direction>,
}

struct Solid<'a>(&'a World);
struct SolidOrOccupied<'a>(&'a World);

const CONFIG: SearchConfig = SearchConfig {
    allow_solid_start: true,
};

const MAX_DEPTH: u32 = 4;

impl<'a> SolidGrid for Solid<'a> {
    fn is_solid(&self, coord: Coord) -> Option<bool> {
        self.0.grid().get(coord).map(|cell| cell.is_solid())
    }
}

impl<'a> SolidGrid for SolidOrOccupied<'a> {
    fn is_solid(&self, coord: Coord) -> Option<bool> {
        self.0.grid().get(coord).map(|cell| cell.is_solid())
    }
}

impl PathfindingContext {
    pub fn new(size: Size) -> Self {
        Self {
            search: SearchContext::new(size),
            bfs: BfsContext::new(size),
            distance_to_player: UniformDistanceMap::new(size, DirectionsCardinal),
            path: Vec::new(),
        }
    }
    pub fn update(&mut self, player_coord: Coord, world: &World) {
        self.bfs
            .populate_uniform_distance_map(
                &Solid(world),
                player_coord,
                Default::default(),
                &mut self.distance_to_player,
            )
            .expect("Failed to update distance to player");
    }
    pub fn direction_towards_player(
        &mut self,
        coord: Coord,
        world: &World,
    ) -> Option<CardinalDirection> {
        let result = self.search.best_search_uniform_distance_map(
            &SolidOrOccupied(world),
            coord,
            CONFIG,
            MAX_DEPTH,
            &self.distance_to_player,
            &mut self.path,
        );
        match result {
            Ok(_) => self.path.iter().next().and_then(|d| d.cardinal()),
            Err(_) => None,
        }
    }
}
