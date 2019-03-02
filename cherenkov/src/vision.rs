use crate::world::*;
use coord_2d::*;
use direction::*;
use grid_2d::*;
use rgb24::*;
use shadowcast::*;

const OMNISCIENT: bool = false;
const AMBIENT_LIGHT_FLOOR: Option<u8> = None;

struct Visibility;

impl InputGrid for Visibility {
    type Grid = World;
    type Opacity = u8;
    fn size(&self, world: &Self::Grid) -> Size {
        world.grid().size()
    }
    fn get_opacity(&self, world: &Self::Grid, coord: Coord) -> Self::Opacity {
        world.opacity(coord)
    }
}

const VISION_DISTANCE_SQUARED: u32 = 60;
const VISION_DISTANCE: vision_distance::Circle =
    vision_distance::Circle::new_squared(VISION_DISTANCE_SQUARED);

#[derive(Clone, Serialize, Deserialize)]
pub struct VisibilityCell {
    last_seen: u64,
    last_lit: u64,
    visible_directions: DirectionBitmap,
    light_colour: Rgb24,
}

#[derive(Clone, Copy)]
pub struct VisibilityState {
    count: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VisibileArea {
    grid: Grid<VisibilityCell>,
    count: u64,
    #[serde(skip)]
    shadowcast: ShadowcastContext<u8>,
}

impl VisibileArea {
    pub fn new(size: Size) -> Self {
        let grid = Grid::new_clone(
            size,
            VisibilityCell {
                last_seen: 0,
                last_lit: 0,
                light_colour: rgb24(0, 0, 0),
                visible_directions: DirectionBitmap::empty(),
            },
        );
        let count = 1;
        let shadowcast = ShadowcastContext::default();
        Self {
            grid,
            count,
            shadowcast,
        }
    }
    pub fn state(&self) -> VisibilityState {
        VisibilityState { count: self.count }
    }
    pub fn iter(&self) -> impl Iterator<Item = &VisibilityCell> {
        self.grid.iter()
    }
    pub fn update(&mut self, player_coord: Coord, world: &World) {
        self.count += 1;
        let count = self.count;
        let grid = &mut self.grid;
        if OMNISCIENT {
            for cell in grid.iter_mut() {
                cell.last_seen = count;
                cell.visible_directions = DirectionBitmap::all();
            }
        } else {
            self.shadowcast.for_each_visible(
                player_coord,
                &Visibility,
                &world,
                VISION_DISTANCE,
                255,
                |coord, direction_bitmap, _visibility| {
                    let cell = grid.get_checked_mut(coord);
                    cell.last_seen = count;
                    cell.visible_directions = direction_bitmap;
                },
            );
        }
        for light in world.lights().iter() {
            self.shadowcast.for_each_visible(
                light.coord(),
                &Visibility,
                &world,
                light.range(),
                255,
                |coord, direction_bitmap, visibility| {
                    let cell = grid.get_checked_mut(coord);
                    if cell.last_seen == count
                        && !(direction_bitmap & cell.visible_directions).is_empty()
                    {
                        if cell.last_lit != count {
                            cell.last_lit = count;
                            cell.light_colour = rgb24(0, 0, 0);
                        }
                        cell.light_colour = cell.light_colour.saturating_add(
                            light
                                .colour_at_coord(coord)
                                .normalised_scalar_mul(visibility),
                        );
                    }
                },
            );
        }
        if let Some(ambient_light_floor) = AMBIENT_LIGHT_FLOOR {
            for cell in grid.iter_mut() {
                cell.last_lit = count;
                cell.light_colour = cell.light_colour.floor(ambient_light_floor);
            }
        }
    }
}

impl VisibilityCell {
    pub fn is_visible(&self, state: VisibilityState) -> bool {
        self.last_seen == state.count
    }
    pub fn is_discovered(&self) -> bool {
        self.last_seen != 0
    }
    pub fn light_colour(&self, state: VisibilityState) -> Rgb24 {
        if self.last_lit == state.count {
            self.light_colour
        } else {
            rgb24(0, 0, 0)
        }
    }
}
