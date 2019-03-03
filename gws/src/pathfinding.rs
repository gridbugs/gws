use crate::world::*;
use direction::*;
use grid_search::*;

#[derive(Serialize, Deserialize)]
pub struct PathfindingContext {
    search: SearchContext<u32>,
    bfs: BfsContext,
    distance_to_player: UniformDistanceMap<u32, DirectionsCardinal>,
}

struct SearchGrid<'a>(&'a World);

impl<'a> SolidGrid for SearchGrid<'a> {
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
        }
    }
    pub fn update(&mut self, player_coord: Coord, world: &World) {
        self.bfs
            .populate_uniform_distance_map(
                &SearchGrid(world),
                player_coord,
                Default::default(),
                &mut self.distance_to_player,
            )
            .expect("Failed to update distance to player");
    }
}
