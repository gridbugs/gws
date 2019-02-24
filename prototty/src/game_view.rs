use cherenkov::*;
use prototty::*;

pub struct GameView;

const FLOOR: ViewCell = ViewCell::new().with_character('.');
const WALL_ABOVE_FLOOR: ViewCell = ViewCell::new()
    .with_character('▀')
    .with_background(Rgb24::new(127, 127, 127));
const WALL_ABOVE_WALL: ViewCell = ViewCell::new().with_character('█');
const PLAYER: ViewCell = ViewCell::new().with_character('@');

impl View<Cherenkov> for GameView {
    fn view<G: ViewGrid>(&mut self, game: &Cherenkov, offset: Coord, depth: i32, grid: &mut G) {
        for (coord, &cell) in game.grid().enumerate() {
            let cell_info = match cell {
                Cell::Floor => FLOOR,
                Cell::Wall => {
                    if let Some(cell_below) = game.grid().get(coord + Coord::new(0, 1)) {
                        match cell_below {
                            Cell::Floor => WALL_ABOVE_FLOOR,
                            Cell::Wall => WALL_ABOVE_WALL,
                        }
                    } else {
                        WALL_ABOVE_FLOOR
                    }
                }
            };
            grid.set_cell(offset + coord, depth, cell_info);
        }
        grid.set_cell(offset + game.player_coord(), depth, PLAYER);
    }
}
