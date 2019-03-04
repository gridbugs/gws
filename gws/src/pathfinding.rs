use crate::world::*;
use direction::*;
use grid_2d::*;
use grid_search::*;

#[derive(Clone, Serialize, Deserialize)]
struct CommitmentCell {
    seq: u64,
    direction: Option<CardinalDirection>,
}

impl CommitmentCell {
    fn new() -> Self {
        Self {
            seq: 0,
            direction: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CommitmentGrid {
    seq: u64,
    grid: Grid<CommitmentCell>,
}

impl CommitmentGrid {
    fn new(size: Size) -> Self {
        let seq = 1;
        let grid = Grid::new_clone(size, CommitmentCell::new());
        Self { seq, grid }
    }
    fn clear(&mut self) {
        self.seq += 1;
    }
    fn commit(&mut self, coord: Coord, direction: CardinalDirection) {
        let cell = self.grid.get_checked_mut(coord);
        cell.seq = self.seq;
        cell.direction = Some(direction);
    }
    fn is_committed(&self, coord: Coord) -> bool {
        self.grid.get_checked(coord).seq == self.seq
    }
    pub fn get_direction_checked(&self, coord: Coord) -> Option<CardinalDirection> {
        let cell = self.grid.get_checked(coord);
        if cell.seq == self.seq {
            cell.direction
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PathfindingContext {
    search: SearchContext<u32>,
    bfs: BfsContext,
    distance_to_player: UniformDistanceMap<u32, DirectionsCardinal>,
    path: Vec<Direction>,
    commitment_grid: CommitmentGrid,
    player_coord: Coord,
    committed_movements: Vec<(EntityId, CardinalDirection)>,
}

struct Solid<'a>(&'a World);
struct SolidOrOccupied<'a> {
    world: &'a World,
    commitment_grid: &'a CommitmentGrid,
}

impl<'a> SolidGrid for Solid<'a> {
    fn is_solid(&self, coord: Coord) -> Option<bool> {
        self.0.grid().get(coord).map(|cell| cell.is_solid())
    }
}

impl<'a> SolidGrid for SolidOrOccupied<'a> {
    fn is_solid(&self, coord: Coord) -> Option<bool> {
        if let Some(cell) = self.world.grid().get(coord) {
            Some(
                cell.is_solid()
                    || cell.contains_npc()
                    || self.commitment_grid.is_committed(coord),
            )
        } else {
            None
        }
    }
}

const CONFIG: SearchConfig = SearchConfig {
    allow_solid_start: true,
};

const MAX_DEPTH: u32 = 4;

impl PathfindingContext {
    pub fn new(size: Size) -> Self {
        Self {
            player_coord: Coord::new(0, 0),
            search: SearchContext::new(size),
            bfs: BfsContext::new(size),
            commitment_grid: CommitmentGrid::new(size),
            distance_to_player: UniformDistanceMap::new(size, DirectionsCardinal),
            committed_movements: Vec::new(),
            path: Vec::new(),
        }
    }
    pub fn commitment_grid(&self) -> &CommitmentGrid {
        &self.commitment_grid
    }
    pub fn update_player_coord(&mut self, player_coord: Coord, world: &World) {
        if player_coord.is_valid(world.grid().size()) {
            self.bfs
                .populate_uniform_distance_map(
                    &Solid(world),
                    player_coord,
                    Default::default(),
                    &mut self.distance_to_player,
                )
                .expect("Failed to update distance to player");
        }
        self.player_coord = player_coord;
        self.commitment_grid.clear();
        self.committed_movements.clear();
    }
    pub fn direction_towards_player(
        &mut self,
        coord: Coord,
        world: &World,
    ) -> Option<CardinalDirection> {
        let result = self.search.best_search_uniform_distance_map(
            &SolidOrOccupied {
                world,
                commitment_grid: &self.commitment_grid,
            },
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
    pub fn commit_to_moving_towards_player(&mut self, id: EntityId, world: &World) {
        let coord = world.entities().get(&id).unwrap().coord();
        if let Some(direction) = self.direction_towards_player(coord, world) {
            let next_coord = coord + direction.coord();
            if next_coord != self.player_coord {
                self.commitment_grid.commit(next_coord, direction);
            }
            self.committed_movements.push((id, direction));
        }
    }
    pub fn committed_movements(&self) -> &[(EntityId, CardinalDirection)] {
        &self.committed_movements
    }
}
