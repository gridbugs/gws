use direction::*;
use gws::*;
use prototty::*;

pub struct GameView;

const FLOOR_BACKGROUND: Rgb24 = rgb24(0, 10, 30);
const FLOOR_FOREGROUND: Rgb24 = rgb24(120, 150, 240);
const GROUND_BACKGROUND: Rgb24 = rgb24(0, 0, 0);
const GROUND_FOREGROUND: Rgb24 = rgb24(255, 255, 255);
const TREE_COLOUR: Rgb24 = rgb24(30, 200, 60);
const STAIRS_COLOUR: Rgb24 = rgb24(220, 100, 50);

const ICE_WALL_TOP_COLOUR: Rgb24 = rgb24(60, 80, 120);
const ICE_WALL_FRONT_COLOUR: Rgb24 = FLOOR_FOREGROUND;
const ICE_WALL_ABOVE_FLOOR: ViewCell = ViewCell::new()
    .with_character('▀')
    .with_foreground(ICE_WALL_TOP_COLOUR)
    .with_background(ICE_WALL_FRONT_COLOUR);
const ICE_WALL_ABOVE_WALL: ViewCell = ViewCell::new()
    .with_character(' ')
    .with_background(ICE_WALL_TOP_COLOUR);

const BRICK_WALL_TOP_COLOUR: Rgb24 = rgb24(50, 20, 10);
const BRICK_WALL_FRONT_COLOUR: Rgb24 = rgb24(220, 170, 100);
const BRICK_WALL_ABOVE_FLOOR: ViewCell = ViewCell::new()
    .with_character('▀')
    .with_foreground(BRICK_WALL_TOP_COLOUR)
    .with_background(BRICK_WALL_FRONT_COLOUR);
const BRICK_WALL_ABOVE_WALL: ViewCell = ViewCell::new()
    .with_character(' ')
    .with_background(BRICK_WALL_TOP_COLOUR);

const STONE_WALL_TOP_COLOUR: Rgb24 = rgb24(60, 60, 60);
const STONE_WALL_FRONT_COLOUR: Rgb24 = rgb24(160, 160, 160);
const STONE_WALL_ABOVE_FLOOR: ViewCell = ViewCell::new()
    .with_character('▀')
    .with_foreground(STONE_WALL_TOP_COLOUR)
    .with_background(STONE_WALL_FRONT_COLOUR);
const STONE_WALL_ABOVE_WALL: ViewCell = ViewCell::new()
    .with_character(' ')
    .with_background(STONE_WALL_TOP_COLOUR);

const BLOCK_TOP_COLOUR: Rgb24 = rgb24(60, 140, 100);
const BLOCK_FRONT_COLOUR: Rgb24 = rgb24(100, 200, 140);
const BLOCK: ViewCell = ViewCell::new()
    .with_character('▀')
    .with_foreground(BLOCK_TOP_COLOUR)
    .with_background(BLOCK_FRONT_COLOUR);

const FLOOR: ViewCell = ViewCell::new()
    .with_character('.')
    .with_foreground(FLOOR_FOREGROUND)
    .with_background(FLOOR_BACKGROUND);
const GROUND: ViewCell = ViewCell::new()
    .with_character('.')
    .with_foreground(GROUND_FOREGROUND)
    .with_background(GROUND_BACKGROUND);
const TREE: ViewCell = ViewCell::new()
    .with_character('♣')
    .with_bold(true)
    .with_foreground(TREE_COLOUR);
const STAIRS: ViewCell = ViewCell::new()
    .with_character('>')
    .with_bold(true)
    .with_foreground(STAIRS_COLOUR);
const PLAYER: ViewCell = ViewCell::new().with_character('@').with_bold(true);

const END: ViewCell = ViewCell::new()
    .with_character('@')
    .with_bold(true)
    .with_foreground(rgb24(200, 0, 255));

const BRUISER_CHAR: char = 'b';
const BRUISER_VIEW_CELL: ViewCell = ViewCell::new()
    .with_bold(true)
    .with_foreground(rgb24(200, 20, 80));

const CASTER_CHAR: char = 'c';
const CASTER_VIEW_CELL: ViewCell = ViewCell::new()
    .with_bold(true)
    .with_foreground(rgb24(30, 200, 80));

const HEALER_CHAR: char = 'h';
const HEALER_VIEW_CELL: ViewCell = ViewCell::new()
    .with_bold(true)
    .with_foreground(rgb24(200, 200, 80));

const ARROW_CHARS: CardinalDirectionTable<char> =
    CardinalDirectionTable::new_array(['↑', '→', '↓', '←']);

const MOVE_VIEW_CELL: ViewCell = ViewCell::new()
    .with_bold(false)
    .with_foreground(rgb24(255, 255, 255));
const ATTACK_VIEW_CELL: ViewCell = ViewCell::new()
    .with_bold(false)
    .with_foreground(rgb24(0, 255, 255));
const HEAL_VIEW_CELL: ViewCell = ViewCell::new()
    .with_bold(true)
    .with_foreground(rgb24(200, 200, 0));

const BLINK0: ViewCell = ViewCell::new()
    .with_character('☼')
    .with_bold(true)
    .with_foreground(rgb24(0, 255, 255));

const BLINK1: ViewCell = ViewCell::new()
    .with_character('*')
    .with_bold(false)
    .with_foreground(rgb24(0, 255, 255));

const SPARK: ViewCell = ViewCell::new()
    .with_character('*')
    .with_bold(false)
    .with_foreground(rgb24(0, 200, 200));

const FLAME: ViewCell = ViewCell::new()
    .with_character('Ψ')
    .with_bold(true)
    .with_foreground(rgb24(255, 120, 0));

const ALTAR: ViewCell = ViewCell::new()
    .with_character('₪')
    .with_bold(true)
    .with_foreground(rgb24(0, 200, 50));

const FOUNTAIN: ViewCell = ViewCell::new()
    .with_character('≈')
    .with_bold(true)
    .with_foreground(rgb24(50, 100, 200));

const SPIKE: ViewCell = ViewCell::new()
    .with_character('▲')
    .with_bold(true)
    .with_foreground(rgb24(100, 200, 140));

const NATURAL_SPIKE: ViewCell = ViewCell::new()
    .with_character('▲')
    .with_bold(true)
    .with_foreground(rgb24(0, 255, 255));

const HEALTH_PICKUP: ViewCell = ViewCell::new()
    .with_character('+')
    .with_bold(true)
    .with_foreground(rgb24(140, 0, 0));

fn npc_view_cell(entity: &Entity) -> ViewCell {
    // TODO messy
    let (ch, view_cell) = match entity.foreground_tile().unwrap() {
        ForegroundTile::Bruiser => (BRUISER_CHAR, BRUISER_VIEW_CELL),
        ForegroundTile::Caster => (CASTER_CHAR, CASTER_VIEW_CELL),
        ForegroundTile::Healer => (HEALER_CHAR, HEALER_VIEW_CELL),
        _ => panic!("not npc"),
    };
    match entity.hit_points().expect("missing hit points").current {
        0 => view_cell.with_character('?'),
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
        BackgroundTile::BrickWall => {
            if let Some(cell_below) = to_render.world.grid().get(coord + Coord::new(0, 1))
            {
                match cell_below.background_tile() {
                    BackgroundTile::BrickWall => BRICK_WALL_ABOVE_WALL,
                    _ => BRICK_WALL_ABOVE_FLOOR,
                }
            } else {
                BRICK_WALL_ABOVE_FLOOR
            }
        }
        BackgroundTile::StoneWall => {
            if let Some(cell_below) = to_render.world.grid().get(coord + Coord::new(0, 1))
            {
                match cell_below.background_tile() {
                    BackgroundTile::StoneWall => STONE_WALL_ABOVE_WALL,
                    _ => STONE_WALL_ABOVE_FLOOR,
                }
            } else {
                STONE_WALL_ABOVE_FLOOR
            }
        }
    };
    // TODO this rule should live somewhere else
    let entity = if cell.contains_npc() || cell.contains_player() {
        cell.entity_iter(to_render.world.entities())
            .find(|e| e.is_npc() || e.is_player())
    } else {
        cell.entity_iter(to_render.world.entities())
            .find(|e| e.foreground_tile().is_some())
    };
    if let Some(entity) = entity {
        if let Some(direction) = entity.taking_damage_in_direction() {
            ViewCell::new()
                .with_character(ARROW_CHARS[direction])
                .with_foreground(rgb24(255, 0, 0))
                .coalesce(view_cell)
        } else if entity.is_npc() {
            let view_cell = if let Some(heal_countdown) = entity.heal_countdown() {
                let ch = heal_countdown.to_string().chars().next().unwrap();
                HEAL_VIEW_CELL.with_character(ch).coalesce(view_cell)
            } else {
                npc_view_cell(entity).coalesce(view_cell)
            };
            let view_cell = if entity.is_frozen() {
                view_cell.with_foreground(rgb24(80, 220, 80))
            } else {
                view_cell
            };
            view_cell
        } else if let Some(foreground_tile) = entity.foreground_tile() {
            match foreground_tile {
                ForegroundTile::Player => PLAYER,
                ForegroundTile::End => END,
                ForegroundTile::Block => BLOCK,
                ForegroundTile::Spike => SPIKE,
                ForegroundTile::NaturalSpike => NATURAL_SPIKE,
                ForegroundTile::Spark => SPARK,
                ForegroundTile::HealthPickup => HEALTH_PICKUP,
                ForegroundTile::Tree => TREE,
                ForegroundTile::Stairs => STAIRS,
                ForegroundTile::Blink0 => BLINK0,
                ForegroundTile::Blink1 => BLINK1,
                ForegroundTile::Flame => FLAME,
                ForegroundTile::Altar => ALTAR,
                ForegroundTile::Fountain => FOUNTAIN,
                _ => panic!(),
            }
            .coalesce(view_cell)
        } else {
            view_cell
        }
    } else {
        if let Some((direction, typ)) = to_render.commitment_grid.get_checked(coord) {
            match typ {
                CommitmentType::Move => MOVE_VIEW_CELL
                    .with_character(ARROW_CHARS[direction])
                    .coalesce(view_cell),
                CommitmentType::Cast => {
                    ATTACK_VIEW_CELL.with_character('*').coalesce(view_cell)
                }
                CommitmentType::Heal(_) => view_cell,
            }
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
            let light_colour = visibility.light_colour(visibility_state);
            if light_colour == grey24(0) {
                continue;
            }
            let mut view_cell = game_view_cell(&to_render, cell, coord);
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
