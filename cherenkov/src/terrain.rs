use super::*;
use crate::world::*;
use coord_2d::*;
use rgb24::*;

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

pub fn from_str(s: &str) -> TerrainDescription {
    fn basic_light(rgb24: Rgb24) -> PackedLight {
        PackedLight::new(rgb24.floor(10), 90, Rational::new(1, 10))
    }
    let terrain_vecs = s
        .split("\n")
        .filter(|s| !s.is_empty())
        .map(|s| s.chars().collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let size = Size::new(terrain_vecs[0].len() as u32, terrain_vecs.len() as u32);
    let mut player_coord = None;
    let mut instructions = Vec::new();
    for (y, row) in terrain_vecs.iter().enumerate() {
        for (x, ch) in row.iter().enumerate() {
            let coord = Coord::new(x as i32, y as i32);
            use Instruction::*;
            match ch {
                '.' => instructions.push(SetBackground(coord, BackgroundTile::Floor)),
                ',' => instructions.push(SetBackground(coord, BackgroundTile::Ground)),
                '#' => instructions.push(SetBackground(coord, BackgroundTile::Wall)),
                '&' => {
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
    }
    TerrainDescription::new(player_coord.unwrap(), size, instructions)
}

pub fn wfc_forrest(size: Size) -> TerrainDescription {
    let player_coord = Coord::new(0, 0);
    let instructions = Vec::new();
    TerrainDescription::new(player_coord, size, instructions)
}
