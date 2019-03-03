use super::*;
use crate::world::*;
use coord_2d::*;
use grid_2d::coord_system::XThenYIter;
use grid_2d::*;
use rand::Rng;
use rgb24::*;
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

fn from_char_grid(grid: Grid<char>) -> TerrainDescription {
    fn basic_light(rgb24: Rgb24) -> PackedLight {
        PackedLight::new(rgb24.floor(10), 90, Rational::new(1, 10))
    }
    let mut player_coord = None;
    let mut instructions = Vec::new();
    for (coord, &ch) in grid.enumerate() {
        use Instruction::*;
        match ch {
            '.' => instructions.push(SetBackground(coord, BackgroundTile::Floor)),
            ',' => instructions.push(SetBackground(coord, BackgroundTile::Ground)),
            '#' | '$' => instructions.push(SetBackground(coord, BackgroundTile::Wall)),
            '&' | '%' => {
                instructions.push(SetBackground(coord, BackgroundTile::Ground));
                instructions.push(AddEntity(
                    coord,
                    PackedEntity {
                        foreground_tile: Some(ForegroundTile::Tree),
                        ..Default::default()
                    },
                ));
            }
            '@' => {
                player_coord = Some(coord);
                instructions.push(SetBackground(coord, BackgroundTile::Floor));
            }
            '1' => instructions.push(AddEntity(
                coord,
                PackedEntity {
                    foreground_tile: None,
                    light: Some(basic_light(rgb24(255, 0, 0))),
                },
            )),
            '2' => instructions.push(AddEntity(
                coord,
                PackedEntity {
                    foreground_tile: None,
                    light: Some(basic_light(rgb24(0, 255, 0))),
                },
            )),
            '3' => instructions.push(AddEntity(
                coord,
                PackedEntity {
                    foreground_tile: None,
                    light: Some(basic_light(rgb24(0, 0, 255))),
                },
            )),
            _ => panic!("unrecognised char"),
        }
    }
    TerrainDescription::new(player_coord.unwrap(), grid.size(), instructions)
}

pub fn from_str(s: &str) -> TerrainDescription {
    from_char_grid(string_to_char_grid(s))
}

pub fn wfc_from_str<R: Rng>(
    output_size: Size,
    s: &str,
    rng: &mut R,
) -> TerrainDescription {
    struct Forbid {
        bottom_right_id: PatternId,
        offset: i32,
    }
    impl ForbidPattern for Forbid {
        fn forbid<W: Wrap, R: Rng>(&mut self, fi: &mut ForbidInterface<W>, rng: &mut R) {
            let output_size = fi.wave_size();
            /*
            for x in 0..(output_size.width() as i32 - 1) {
                let coord = Coord::new(x, output_size.height() as i32 - self.offset);
                fi.forbid_all_patterns_except(coord, self.bottom_id, rng)
                    .unwrap();
            }
            for y in 0..(output_size.height() as i32 - 1) {
                let coord = Coord::new(output_size.width() as i32 - self.offset, y);
                fi.forbid_all_patterns_except(coord, self.right_id, rng)
                    .unwrap();
            }*/
            let bottom_right_coord = Coord::new(
                output_size.width() as i32 - self.offset,
                output_size.height() as i32 - self.offset,
            );
            fi.forbid_all_patterns_except(bottom_right_coord, self.bottom_right_id, rng)
                .unwrap();
            for coord in XThenYIter::new(output_size) {
                if coord != bottom_right_coord {
                    fi.forbid_pattern(coord, self.bottom_right_id, rng).unwrap();
                }
            }
        }
    }
    let grid = string_to_char_grid(s);
    let input_size = grid.size();
    let pattern_size = 3;
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
    let bottom_coord =
        Coord::new(0, input_size.height() as i32 - bottom_right_offset as i32);
    let right_coord =
        Coord::new(input_size.width() as i32 - bottom_right_offset as i32, 0);
    let bottom_right_id = *id_grid
        .get_checked(bottom_right_coord)
        .get(Orientation::Original)
        .unwrap();
    let bottom_id = *id_grid
        .get_checked(bottom_coord)
        .get(Orientation::Original)
        .unwrap();
    let right_id = *id_grid
        .get_checked(right_coord)
        .get(Orientation::Original)
        .unwrap();
    for &id in &[bottom_right_id, bottom_id, right_id] {
        overlapping_patterns.pattern_mut(id).clear_count();
    }
    let global_stats = overlapping_patterns.global_stats();
    let mut wave = Wave::new(output_size);
    let mut wfc_context = Context::new();
    let forbid = Forbid {
        bottom_right_id,
        offset: bottom_right_offset as i32,
    };
    let mut run =
        RunBorrow::new_forbid(&mut wfc_context, &mut wave, &global_stats, forbid, rng);
    run.collapse_retrying(Forever, rng);
    let mut output_grid = Grid::new_grid_map_ref(wave.grid(), |wave_cell| {
        let pattern_id = wave_cell.chosen_pattern_id().unwrap();
        *overlapping_patterns.pattern_top_left_value(pattern_id)
    });
    *output_grid.get_checked_mut(Coord::new(0, 0)) = '@';
    from_char_grid(output_grid)
}
