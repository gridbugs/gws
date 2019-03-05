use direction::*;
use gws::*;
use prototty::*;

pub struct GameView;

const FLOOR_BACKGROUND: Rgb24 = rgb24(0, 10, 30);
const FLOOR_FOREGROUND: Rgb24 = rgb24(120, 150, 240);
const GROUND_BACKGROUND: Rgb24 = rgb24(2, 20, 5);
const GROUND_FOREGROUND: Rgb24 = rgb24(255, 255, 255);
const ICE_WALL_TOP_COLOUR: Rgb24 = rgb24(60, 80, 120);
const ICE_WALL_FRONT_COLOUR: Rgb24 = FLOOR_FOREGROUND;
const TREE_COLOUR: Rgb24 = rgb24(30, 200, 60);
const STAIRS_COLOUR: Rgb24 = rgb24(220, 100, 50);

const FLOOR: ViewCell = ViewCell::new()
    .with_character('.')
    .with_foreground(FLOOR_FOREGROUND)
    .with_background(FLOOR_BACKGROUND);
const GROUND: ViewCell = ViewCell::new()
    .with_character('.')
    .with_foreground(GROUND_FOREGROUND)
    .with_background(GROUND_BACKGROUND);
const ICE_WALL_ABOVE_FLOOR: ViewCell = ViewCell::new()
    .with_character('▀')
    .with_foreground(ICE_WALL_TOP_COLOUR)
    .with_background(ICE_WALL_FRONT_COLOUR);
const ICE_WALL_ABOVE_WALL: ViewCell = ViewCell::new()
    .with_character('█')
    .with_foreground(ICE_WALL_TOP_COLOUR)
    .with_background(ICE_WALL_FRONT_COLOUR);
const TREE: ViewCell = ViewCell::new()
    .with_character('♣')
    .with_bold(true)
    .with_foreground(TREE_COLOUR);
const STAIRS: ViewCell = ViewCell::new()
    .with_character('>')
    .with_bold(true)
    .with_foreground(STAIRS_COLOUR);
const PLAYER: ViewCell = ViewCell::new().with_character('@').with_bold(true);

const DEMON_CHAR: char = 'd';
const DEMON_VIEW_CELL: ViewCell = ViewCell::new()
    .with_bold(true)
    .with_foreground(rgb24(30, 200, 80));

const ARROW_CHARS: CardinalDirectionTable<char> =
    CardinalDirectionTable::new_array(['↑', '→', '↓', '←']);

fn npc_view_cell(entity: &Entity) -> ViewCell {
    let (ch, view_cell) = match entity.foreground_tile().unwrap() {
        ForegroundTile::Demon => (DEMON_CHAR, DEMON_VIEW_CELL),
        _ => panic!("not npc"),
    };
    match entity.hit_points().expect("missing hit points").num {
        1 => view_cell.with_character(ch),
        2 => view_cell.with_character(ch.to_uppercase().next().unwrap()),
        3 => view_cell
            .with_character(ch.to_uppercase().next().unwrap())
            .with_underline(true),
        _ => panic!("unexpected npc health"),
    }
}

fn light_view_cell(view_cell: &mut ViewCell, light_colour: Rgb24) {
    if let Some(foreground) = view_cell.foreground.as_mut() {
        *foreground = foreground.normalised_mul(light_colour);
    }
    if let Some(background) = view_cell.background.as_mut() {
        *background = background.normalised_mul(light_colour);
    }
}

fn sub_light_view_cell(view_cell: &mut ViewCell, light_colour: Rgb24) {
    if let Some(foreground) = view_cell.foreground.as_mut() {
        *foreground = foreground.saturating_sub(light_colour);
    }
    if let Some(background) = view_cell.background.as_mut() {
        *background = background.saturating_sub(light_colour);
    }
}

fn game_view_cell(to_render: &ToRender, cell: &WorldCell, coord: Coord) -> ViewCell {
    let view_cell = match cell.background_tile() {
        BackgroundTile::Floor => FLOOR,
        BackgroundTile::Ground => GROUND,
        BackgroundTile::IceWall => {
            if let Some(cell_below) = to_render.world.grid().get(coord + Coord::new(0, 1))
            {
                match cell_below.background_tile() {
                    BackgroundTile::IceWall => ICE_WALL_ABOVE_WALL,
                    _ => ICE_WALL_ABOVE_FLOOR,
                }
            } else {
                ICE_WALL_ABOVE_FLOOR
            }
        }
    };
    if let Some(entity) = cell.entity_iter(to_render.world.entities()).next() {
        if let Some(direction) = entity.taking_damage_in_direction() {
            ViewCell::new()
                .with_character(ARROW_CHARS[direction])
                .with_foreground(rgb24(255, 0, 0))
                .coalesce(view_cell)
        } else if entity.is_npc() {
            npc_view_cell(entity).coalesce(view_cell)
        } else if let Some(foreground_tile) = entity.foreground_tile() {
            match foreground_tile {
                ForegroundTile::Player => PLAYER,
                ForegroundTile::Tree => TREE,
                ForegroundTile::Stairs => STAIRS,
                _ => panic!(),
            }
            .coalesce(view_cell)
        } else {
            view_cell
        }
    } else {
        if let Some(direction) = to_render.commitment_grid.get_direction_checked(coord) {
            ViewCell::new()
                .with_character(ARROW_CHARS[direction])
                .coalesce(view_cell)
        } else {
            view_cell
        }
    }
}

impl View<Gws> for GameView {
    fn view<G: ViewGrid>(&mut self, game: &Gws, offset: Coord, depth: i32, grid: &mut G) {
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
            let mut view_cell = game_view_cell(&to_render, cell, coord);
            let light_colour = visibility.light_colour(visibility_state);
            light_view_cell(&mut view_cell, light_colour);
            grid.set_cell(offset + coord, depth, view_cell);
        }
    }
}

pub struct DeathGameView;
impl View<Gws> for DeathGameView {
    fn view<G: ViewGrid>(&mut self, game: &Gws, offset: Coord, depth: i32, grid: &mut G) {
        let to_render = game.to_render();
        let visibility_state = to_render.visible_area.state();
        for ((coord, cell), visibility) in to_render
            .world
            .grid()
            .enumerate()
            .zip(to_render.visible_area.iter())
        {
            let mut view_cell = game_view_cell(&to_render, cell, coord);
            if visibility.is_visible(visibility_state) {
                let light_colour = visibility
                    .light_colour(visibility_state)
                    .saturating_add(rgb24(128, 0, 0));
                light_view_cell(&mut view_cell, light_colour);
                sub_light_view_cell(&mut view_cell, rgb24(0, 200, 200));
            } else {
                light_view_cell(&mut view_cell, rgb24(64, 0, 0));
            };
            grid.set_cell(offset + coord, depth, view_cell);
        }
    }
}
