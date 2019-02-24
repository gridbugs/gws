use cherenkov::*;
use prototty::*;

pub struct GameView;

const FLOOR_BACKGROUND: Rgb24 = Rgb24::new(0, 0, 127);
const FLOOR_FOREGROUND: Rgb24 = Rgb24::new(255, 255, 255);
const WALL_TOP_COLOUR: Rgb24 = Rgb24::new(200, 128, 0);
const WALL_FRONT_COLOUR: Rgb24 = Rgb24::new(200, 50, 0);

const FLOOR: ViewCell = ViewCell::new()
    .with_character('.')
    .with_foreground(FLOOR_FOREGROUND)
    .with_background(FLOOR_BACKGROUND);
const WALL_ABOVE_FLOOR: ViewCell = ViewCell::new()
    .with_character('▀')
    .with_foreground(WALL_TOP_COLOUR)
    .with_background(WALL_FRONT_COLOUR);
const WALL_ABOVE_WALL: ViewCell = ViewCell::new()
    .with_character('█')
    .with_foreground(WALL_TOP_COLOUR)
    .with_background(WALL_FRONT_COLOUR);
const PLAYER: ViewCell = ViewCell::new().with_character('@');

const LIGHT_DIMINISH_DAMPEN: u32 = 4;

#[derive(Clone, Copy)]
struct Rational {
    num: u32,
    denom: u32,
}

fn mult_channel(c: u8, by: Rational) -> u8 {
    ((c as u32 * by.num) / by.denom) as u8
}

fn mult_rgb24(Rgb24 { red, green, blue }: Rgb24, by: Rational) -> Rgb24 {
    Rgb24 {
        red: mult_channel(red, by),
        green: mult_channel(green, by),
        blue: mult_channel(blue, by),
    }
}

fn mult_cell_info(cell_info: &mut ViewCell, by: Rational) {
    if let Some(foreground) = cell_info.foreground.as_mut() {
        *foreground = mult_rgb24(*foreground, by);
    }
    if let Some(background) = cell_info.background.as_mut() {
        *background = mult_rgb24(*background, by);
    }
}

impl View<Cherenkov> for GameView {
    fn view<G: ViewGrid>(&mut self, game: &Cherenkov, offset: Coord, depth: i32, grid: &mut G) {
        let to_render = game.to_render();
        let visibility_state = to_render.visible_area.state();
        for ((coord, &cell), visibility) in to_render
            .grid
            .enumerate()
            .zip(to_render.visible_area.iter())
        {
            if !visibility.is_visible(visibility_state) {
                continue;
            }
            let mut cell_info = match cell {
                Cell::Floor => FLOOR,
                Cell::Wall => {
                    if let Some(cell_below) = to_render.grid.get(coord + Coord::new(0, 1)) {
                        match cell_below {
                            Cell::Floor => WALL_ABOVE_FLOOR,
                            Cell::Wall => WALL_ABOVE_WALL,
                        }
                    } else {
                        WALL_ABOVE_FLOOR
                    }
                }
            };
            let square_distance = {
                let d = to_render.player_coord - coord;
                d.x * d.x + d.y * d.y
            } as u32;
            let dampened_square_distance = (square_distance / LIGHT_DIMINISH_DAMPEN).max(1);
            mult_cell_info(
                &mut cell_info,
                Rational {
                    num: 1,
                    denom: dampened_square_distance,
                },
            );
            grid.set_cell(offset + coord, depth, cell_info);
        }
        grid.set_cell(offset + to_render.player_coord, depth, PLAYER);
    }
}
