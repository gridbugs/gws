use cherenkov::*;
use prototty::*;

pub struct MapView;

const FLOOR: ViewCell = ViewCell::new()
    .with_character('.')
    .with_foreground(colours::WHITE)
    .with_background(colours::BLACK);
const WALL: ViewCell = ViewCell::new()
    .with_character('#')
    .with_foreground(colours::WHITE)
    .with_background(colours::BLACK);
const PLAYER: ViewCell = ViewCell::new()
    .with_character('@')
    .with_foreground(colours::WHITE)
    .with_background(colours::BLACK);

impl View<Cherenkov> for MapView {
    fn view<G: ViewGrid>(&mut self, game: &Cherenkov, offset: Coord, depth: i32, grid: &mut G) {
        let to_render = game.to_render();
        for ((coord, cell), visibility) in to_render
            .world
            .grid()
            .enumerate()
            .zip(to_render.visible_area.iter())
        {
            if !visibility.is_discovered() {
                continue;
            }
            let cell_info = match cell.base() {
                WorldCellBase::Floor => FLOOR,
                WorldCellBase::Wall => WALL,
            };
            grid.set_cell(offset + coord, depth, cell_info);
        }
        grid.set_cell(offset + to_render.player_coord, depth, PLAYER);
    }
}
