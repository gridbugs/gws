use cherenkov::*;
use prototty::*;

pub struct GameView;

const FLOOR_BACKGROUND: Rgb24 = rgb24(0, 0, 127);
const FLOOR_FOREGROUND: Rgb24 = rgb24(255, 255, 255);
const WALL_TOP_COLOUR: Rgb24 = rgb24(255, 255, 0);
const WALL_FRONT_COLOUR: Rgb24 = rgb24(255, 50, 0);

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
const PLAYER: ViewCell = ViewCell::new().with_character('@').with_bold(true);

fn light_view_cell(view_cell: &mut ViewCell, light_colour: Rgb24) {
    if let Some(foreground) = view_cell.foreground.as_mut() {
        *foreground = foreground.normalised_mul(light_colour);
    }
    if let Some(background) = view_cell.background.as_mut() {
        *background = background.normalised_mul(light_colour);
    }
}

impl View<Cherenkov> for GameView {
    fn view<G: ViewGrid>(
        &mut self,
        game: &Cherenkov,
        offset: Coord,
        depth: i32,
        grid: &mut G,
    ) {
        let to_render = game.to_render();
        let visibility_state = to_render.visible_area.state();
        for ((coord, cell), visibility) in to_render
            .world
            .grid()
            .enumerate()
            .zip(to_render.visible_area.iter())
        {
            if !visibility.is_visible(visibility_state) {
                continue;
            }
            let view_cell = match cell.background_tile() {
                BackgroundTile::Floor => FLOOR,
                BackgroundTile::Wall => {
                    if let Some(cell_below) =
                        to_render.world.grid().get(coord + Coord::new(0, 1))
                    {
                        match cell_below.background_tile() {
                            BackgroundTile::Floor => WALL_ABOVE_FLOOR,
                            BackgroundTile::Wall => WALL_ABOVE_WALL,
                        }
                    } else {
                        WALL_ABOVE_FLOOR
                    }
                }
            };
            let mut view_cell = if let Some(foreground_tile) =
                cell.foreground_tiles(to_render.world.entities()).next()
            {
                match foreground_tile {
                    ForegroundTile::Player => PLAYER,
                }
                .coalesce(view_cell)
            } else {
                view_cell
            };
            let light_colour = visibility.light_colour(visibility_state);
            light_view_cell(&mut view_cell, light_colour);
            grid.set_cell(offset + coord, depth, view_cell);
        }
    }
}
