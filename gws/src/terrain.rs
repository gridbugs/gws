use super::*;
use crate::world::*;
use coord_2d::*;
use grid_2d::coord_system::XThenYIter;
use grid_2d::*;
use hashbrown::HashSet;
use rand::Rng;
use rgb24::*;
use std::collections::VecDeque;
use std::num::NonZeroU32;
use wfc::overlapping::*;
use wfc::retry::*;
use wfc::*;

pub struct TerrainDescription {
    pub player_coord: Coord,
    pub size: Size,
    pub instructions: Vec<Instruction>,
}

impl TerrainDescription {
    pub fn new(player_coord: Coord, size: Size, instructions: Vec<Instruction>) -> Self {
        Self {
            player_coord,
            size,
            instructions,
        }
    }
}

fn string_to_char_grid(s: &str) -> Grid<char> {
    let v = s
        .split("\n")
        .filter(|s| !s.is_empty())
        .map(|s| s.chars().collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let size = Size::new(v[0].len() as u32, v.len() as u32);
    Grid::new_fn(size, |Coord { x, y }| v[y as usize][x as usize])
}

#[derive(Debug, Clone, Copy)]
enum Base {
    Floor,
    Ground,
    Tree,
    Wall,
}

#[derive(Debug, Clone, Copy)]
enum Contents {
    Player,
    Light(Rgb24),
}

#[derive(Debug, Clone, Copy)]
struct Cell {
    base: Base,
    contents: Option<Contents>,
}

impl Cell {
    fn new(base: Base) -> Self {
        Self {
            base,
            contents: None,
        }
    }
    fn with_contents(self, contents: Contents) -> Self {
        Self {
            contents: Some(contents),
            ..self
        }
    }
}

fn cell_grid_to_terrain_description(grid: &Grid<Cell>) -> TerrainDescription {
    fn basic_light(rgb24: Rgb24) -> PackedLight {
        PackedLight::new(rgb24.floor(10), 90, Rational::new(1, 10))
    }
    let mut player_coord = None;
    let mut instructions = Vec::new();
    for (coord, &cell) in grid.enumerate() {
        use Instruction::*;
        if let Some(contents) = cell.contents {
            match contents {
                Contents::Player => {
                    player_coord = Some(coord);
                }
                Contents::Light(colour) => instructions.push(AddEntity(
                    coord,
                    PackedEntity {
                        foreground_tile: None,
                        light: Some(basic_light(colour)),
                    },
                )),
            }
        }
        match cell.base {
            Base::Floor => instructions.push(SetBackground(coord, BackgroundTile::Floor)),
            Base::Ground => {
                instructions.push(SetBackground(coord, BackgroundTile::Ground))
            }
            Base::Wall => instructions.push(SetBackground(coord, BackgroundTile::Wall)),
            Base::Tree => {
                instructions.push(SetBackground(coord, BackgroundTile::Ground));
                instructions.push(AddEntity(
                    coord,
                    PackedEntity {
                        foreground_tile: Some(ForegroundTile::Tree),
                        ..Default::default()
                    },
                ));
            }
        }
    }
    TerrainDescription::new(player_coord.unwrap(), grid.size(), instructions)
}

fn char_to_base(ch: char) -> Option<Base> {
    match ch {
        '.' => Some(Base::Floor),
        ',' => Some(Base::Ground),
        '#' | '$' | '?' => Some(Base::Wall),
        '&' | '%' => Some(Base::Tree),
        _ => None,
    }
}

fn char_to_cell(ch: char) -> Option<Cell> {
    if let Some(base) = char_to_base(ch) {
        Some(Cell::new(base))
    } else {
        match ch {
            '@' => Some(Cell::new(Base::Floor).with_contents(Contents::Player)),
            '1' => Some(
                Cell::new(Base::Floor).with_contents(Contents::Light(rgb24(255, 0, 0))),
            ),
            '2' => Some(
                Cell::new(Base::Floor).with_contents(Contents::Light(rgb24(0, 255, 0))),
            ),
            '3' => Some(
                Cell::new(Base::Floor).with_contents(Contents::Light(rgb24(0, 0, 255))),
            ),
            _ => None,
        }
    }
}

fn char_grid_to_base_grid(char_grid: &Grid<char>) -> Grid<Base> {
    Grid::new_grid_map_ref(char_grid, |&ch| {
        char_to_base(ch).expect(&format!("unrecognised char: {}", ch))
    })
}

fn char_grid_to_cell_grid(char_grid: &Grid<char>) -> Grid<Cell> {
    Grid::new_grid_map_ref(char_grid, |&ch| {
        char_to_cell(ch).expect(&format!("unrecognised char: {}", ch))
    })
}

fn char_grid_to_terrain_description(grid: &Grid<char>) -> TerrainDescription {
    cell_grid_to_terrain_description(&char_grid_to_cell_grid(grid))
}

fn base_grid_to_default_cell_grid(base_grid: &Grid<Base>) -> Grid<Cell> {
    Grid::new_grid_map_ref(&base_grid, |base| Cell::new(*base))
}

pub fn from_str(s: &str) -> TerrainDescription {
    char_grid_to_terrain_description(&string_to_char_grid(s))
}

fn binary_distance_map<T, F>(grid: &Grid<T>, mut zero: F) -> Grid<usize>
where
    F: FnMut(&T) -> bool,
{
    let mut queue = VecDeque::new();
    let mut output = Grid::new_clone(grid.size(), None);
    for (coord, cell) in grid.enumerate() {
        if zero(cell) {
            queue.push_back(coord);
            *output.get_checked_mut(coord) = Some(0);
        }
    }
    while let Some(coord) = queue.pop_front() {
        let next_count = output.get_checked(coord).unwrap() + 1;
        for direction in CardinalDirections {
            let next_coord = coord + direction.coord();
            if let Some(cell) = output.get_mut(next_coord) {
                if cell.is_none() {
                    *cell = Some(next_count);
                    queue.push_back(coord);
                }
            }
        }
    }
    Grid::new_grid_map(output, |cell| cell.unwrap())
}

fn fill<T, F>(grid: &Grid<T>, start: Coord, mut can_enter: F) -> HashSet<Coord>
where
    F: FnMut(&T) -> bool,
{
    let mut filled = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(start);
    filled.insert(start);
    while let Some(coord) = queue.pop_front() {
        for direction in CardinalDirections {
            let next_coord = coord + direction.coord();
            if let Some(cell) = grid.get(next_coord) {
                if can_enter(cell) && !filled.contains(&next_coord) {
                    queue.push_back(next_coord);
                    filled.insert(next_coord);
                }
            }
        }
    }
    filled
}

fn classify<T, F>(grid: &Grid<T>, mut can_enter: F) -> Vec<HashSet<Coord>>
where
    F: FnMut(&T) -> bool,
{
    let mut visited = HashSet::new();
    let mut ret = Vec::new();
    for (coord, cell) in grid.enumerate() {
        if can_enter(cell) && !visited.contains(&coord) {
            let area = fill(grid, coord, &mut can_enter);
            for &coord in area.iter() {
                visited.insert(coord);
            }
            ret.push(area);
        }
    }
    ret.sort_by(|a, b| a.len().cmp(&b.len()));
    ret
}

struct BadLevel;
const MIN_ACCESSIBLE_CELLS: usize = 500;

fn populate_base_grid(base_grid: &Grid<Base>) -> Result<Grid<Cell>, BadLevel> {
    let mut areas = classify(base_grid, |&base| match base {
        Base::Floor | Base::Ground => true,
        Base::Wall | Base::Tree => false,
    });
    let (to_keep, to_fill) = if let Some(last) = areas.pop() {
        (last, areas)
    } else {
        return Err(BadLevel);
    };
    if to_keep.len() < MIN_ACCESSIBLE_CELLS {
        return Err(BadLevel);
    }
    let mut cell_grid = base_grid_to_default_cell_grid(base_grid);
    for &coord in to_fill.iter().flat_map(|a| a.iter()) {
        cell_grid.get_checked_mut(coord).base = Base::Wall;
    }
    Ok(cell_grid)
}

fn wfc_ice_cave_base_grid<R: Rng>(output_size: Size, rng: &mut R) -> Grid<Base> {
    struct Forbid {
        bottom_right_id: PatternId,
        ids_to_forbid_bottom_right: HashSet<PatternId>,
        ids_to_forbid_centre: HashSet<PatternId>,
        offset: i32,
    }
    impl ForbidPattern for Forbid {
        fn forbid<W: Wrap, R: Rng>(&mut self, fi: &mut ForbidInterface<W>, rng: &mut R) {
            let output_size = fi.wave_size();
            let bottom_right_coord = Coord::new(
                output_size.width() as i32 - self.offset,
                output_size.height() as i32 - self.offset,
            );
            fi.forbid_all_patterns_except(bottom_right_coord, self.bottom_right_id, rng)
                .unwrap();
            for coord in XThenYIter::new(output_size) {
                let delta = coord - bottom_right_coord;
                if delta.magnitude2() > 2 {
                    for &id in self.ids_to_forbid_bottom_right.iter() {
                        fi.forbid_pattern(coord, id, rng).unwrap();
                    }
                }
                let pad = 6;
                if coord.x > pad
                    && coord.y > pad
                    && coord.x < output_size.width() as i32 - pad
                    && coord.y < output_size.height() as i32 - pad
                {
                    for &id in self.ids_to_forbid_centre.iter() {
                        fi.forbid_pattern(coord, id, rng).unwrap();
                    }
                }
            }
        }
    }
    let pattern_size = 4;
    let grid = string_to_char_grid(include_str!("wfc_ice_cave.txt"));
    let input_size = grid.size();
    // we will discard the bottom row and right column
    let virtual_output_size = output_size + Size::new(1, 1);
    let mut overlapping_patterns = OverlappingPatterns::new(
        grid,
        NonZeroU32::new(pattern_size).unwrap(),
        &orientation::ALL,
    );
    let id_grid = overlapping_patterns.id_grid();
    let bottom_right_offset = pattern_size - (pattern_size / 2);
    let bottom_right_coord = Coord::new(
        input_size.width() as i32 - bottom_right_offset as i32,
        input_size.height() as i32 - bottom_right_offset as i32,
    );
    let bottom_right_ids = id_grid
        .get_checked(bottom_right_coord)
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    let top_left_ids = [
        Coord::new(20, 0),
        Coord::new(5, 26),
        Coord::new(22, 54),
        Coord::new(41, 53),
        Coord::new(33, 56),
    ]
    .iter()
    .flat_map(|&coord| id_grid.get_checked(coord).iter().cloned())
    .collect::<HashSet<_>>();
    for &empty_id in id_grid.get_checked(Coord::new(8, 8)).iter() {
        overlapping_patterns.pattern_mut(empty_id).clear_count();
    }
    let bottom_right_id = *id_grid
        .get_checked(bottom_right_coord)
        .get(Orientation::Original)
        .unwrap();
    bottom_right_ids.iter().for_each(|&pattern_id| {
        overlapping_patterns.pattern_mut(pattern_id).clear_count();
    });
    let global_stats = overlapping_patterns.global_stats();
    let mut wave = Wave::new(virtual_output_size);
    let mut context = Context::new();
    let forbid = Forbid {
        bottom_right_id,
        ids_to_forbid_bottom_right: bottom_right_ids,
        ids_to_forbid_centre: top_left_ids,
        offset: bottom_right_offset as i32,
    };
    let mut run =
        RunBorrow::new_forbid(&mut context, &mut wave, &global_stats, forbid, rng);
    run.collapse_retrying(NumTimes(10), rng).unwrap();
    let mut output_grid = Grid::new_fn(output_size, |coord| {
        let pattern_id = wave.grid().get_checked(coord).chosen_pattern_id().unwrap();
        *overlapping_patterns.pattern_top_left_value(pattern_id)
    });
    char_grid_to_base_grid(&output_grid)
}

pub fn wfc_ice_cave<R: Rng>(output_size: Size, rng: &mut R) -> TerrainDescription {
    let mut cell_grid = loop {
        let base_grid = wfc_ice_cave_base_grid(output_size, rng);
        if let Ok(cell_grid) = populate_base_grid(&base_grid) {
            break cell_grid;
        }
    };
    cell_grid.get_checked_mut(Coord::new(0, 0)).contents = Some(Contents::Player);
    cell_grid_to_terrain_description(&cell_grid)
}
