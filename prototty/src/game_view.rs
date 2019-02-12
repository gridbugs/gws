use cherenkov::*;
use prototty::*;

pub struct GameView;

impl View<Cherenkov> for GameView {
    fn view<G: ViewGrid>(&mut self, game: &Cherenkov, offset: Coord, depth: i32, grid: &mut G) {
        for (coord, &cell) in game.grid().enumerate() {
            let cell_info = match cell {
                Cell::Floor => ViewCell::new().with_character('.'),
            };
            grid.set_cell(offset + coord, depth, cell_info);
        }
        let player_cell_info = ViewCell::new().with_character('@');
        grid.set_cell(offset + game.player(), depth, player_cell_info);
    }
}
