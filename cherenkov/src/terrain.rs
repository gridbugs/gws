use super::*;
use crate::world::*;
use coord_2d::*;
use rgb24::*;

pub enum Instruction {
    SetBackground(Coord, BackgroundTile),
    AddEntity(Coord, PackedEntity),
}

pub fn from_str(s: &str) -> (Size, Vec<Instruction>) {
    fn basic_light(rgb24: Rgb24) -> PackedLight {
        PackedLight::new(rgb24.floor(10), 90, Rational::new(1, 10))
    }
    let terrain_vecs = s
        .split("\n")
        .filter(|s| !s.is_empty())
        .map(|s| s.chars().collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let size = Size::new(terrain_vecs[0].len() as u32, terrain_vecs.len() as u32);
    let instructions = terrain_vecs
        .iter()
        .enumerate()
        .flat_map(|(y, row)| {
            row.iter().enumerate().flat_map(move |(x, ch)| {
                let coord = Coord::new(x as i32, y as i32);
                use Instruction::*;
                let instructions = match ch {
                    '.' => vec![SetBackground(coord, BackgroundTile::Floor)],
                    ',' => vec![SetBackground(coord, BackgroundTile::Ground)],
                    '#' => vec![SetBackground(coord, BackgroundTile::Wall)],
                    '&' => vec![
                        SetBackground(coord, BackgroundTile::Ground),
                        AddEntity(
                            coord,
                            PackedEntity {
                                foreground_tile: Some(ForegroundTile::Tree),
                                ..Default::default()
                            },
                        ),
                    ],
                    '@' => vec![
                        SetBackground(coord, BackgroundTile::Floor),
                        AddEntity(coord, PackedEntity::player()),
                    ],
                    '1' => vec![AddEntity(
                        coord,
                        PackedEntity {
                            is_player: false,
                            foreground_tile: None,
                            light: Some(basic_light(rgb24(255, 0, 0))),
                        },
                    )],
                    '2' => vec![AddEntity(
                        coord,
                        PackedEntity {
                            is_player: false,
                            foreground_tile: None,
                            light: Some(basic_light(rgb24(0, 255, 0))),
                        },
                    )],
                    '3' => vec![AddEntity(
                        coord,
                        PackedEntity {
                            is_player: false,
                            foreground_tile: None,
                            light: Some(basic_light(rgb24(0, 0, 255))),
                        },
                    )],
                    _ => panic!("unrecognised char"),
                };
                instructions.into_iter()
            })
        })
        .collect();
    (size, instructions)
}
