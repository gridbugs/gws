use crate::game_view::*;
use grid_2d::coord_system::XThenYIter;
use gws::*;
use prototty::*;

pub struct UiView<V: View<Gws>>(pub V);

const STATUS_SIZE_X: i32 = 11;

const STATUS_OFFSET: Coord = Coord::new(1, 1);
const GAME_OFFSET: Coord = Coord {
    x: STATUS_OFFSET.x + STATUS_SIZE_X + 1,
    y: STATUS_OFFSET.y,
};

const TOP_TEXT_OFFSET: Coord = GAME_OFFSET;
const GAME_SIZE: Coord = Coord::new(60, 40);
const CARDS_OFFSET: Coord = Coord {
    x: STATUS_OFFSET.x,
    y: GAME_OFFSET.y + GAME_SIZE.y + 1,
};
const CARD_SIZE: Coord = Coord::new(8, 10);
const CARD_PADDING_X: i32 = 1;

const MAX_NUM_CARDS: usize = 8;

fn test_cards() -> Vec<Option<Card>> {
    vec![
        Some(Card {
            title: "Bump".to_string(),
            description: "Attack adjacent square for 1 damage".to_string(),
            background: rgb24(20, 0, 0),
        }),
        Some(Card {
            title: "Blink".to_string(),
            description: "Teleport to selected square".to_string(),
            background: rgb24(0, 20, 0),
        }),
        None,
        None,
        Some(Card {
            title: "Blink".to_string(),
            description: "Teleport to selected square".to_string(),
            background: rgb24(0, 20, 0),
        }),
        None,
    ]
}

struct StatusView;

impl View<Gws> for StatusView {
    fn view<G: ViewGrid>(&mut self, game: &Gws, offset: Coord, depth: i32, grid: &mut G) {
        let to_render = game.to_render();
        let player_hit_points = to_render.player.hit_points().unwrap();
        let health_colour = if player_hit_points.current <= 1 {
            rgb24(255, 0, 0)
        } else {
            grey24(255)
        };
        StringView.view("Health:", offset, depth, grid);
        RichStringView::with_info(
            TextInfo::default().bold().foreground_colour(health_colour),
        )
        .view(
            &format!("{}/{}", player_hit_points.current, player_hit_points.max),
            offset + Coord::new(0, 1),
            depth,
            grid,
        );
    }
}

impl<V: View<Gws>> View<Gws> for UiView<V> {
    fn view<G: ViewGrid>(&mut self, game: &Gws, offset: Coord, depth: i32, grid: &mut G) {
        self.0.view(game, offset + GAME_OFFSET, depth, grid);
        StatusView.view(game, offset + STATUS_OFFSET, depth, grid);
        CardAreaView.view(&test_cards(), offset + CARDS_OFFSET, depth, grid);
    }
}

struct Card {
    title: String,
    description: String,
    background: Rgb24,
}

struct CardView;
struct CardAreaView;

impl View<[Option<Card>]> for CardAreaView {
    fn view<G: ViewGrid>(
        &mut self,
        cards: &[Option<Card>],
        offset: Coord,
        depth: i32,
        grid: &mut G,
    ) {
        for i in 0..MAX_NUM_CARDS {
            let offset_x = i as i32 * (CARD_SIZE.x + CARD_PADDING_X);
            StringView.view(
                &format!("{}.", i + 1),
                offset + Coord::new(offset_x + 3, 0),
                depth,
                grid,
            );
            let coord = offset + Coord::new(offset_x, 1);

            if let Some(maybe_card) = cards.get(i) {
                if let Some(card) = maybe_card.as_ref() {
                    CardView.view(card, coord, depth, grid);
                } else {
                    empty_card_view(coord, depth, grid);
                }
            } else {
                locked_card_view(coord, depth, grid);
            }
        }
    }
}

struct LockedView;
impl View<Size> for LockedView {
    fn view<G: ViewGrid>(
        &mut self,
        _size: &Size,
        offset: Coord,
        depth: i32,
        grid: &mut G,
    ) {
        RichStringView::with_info(TextInfo::default().foreground_colour(grey24(128)))
            .view("Locked", offset + Coord::new(0, 4), depth, grid);
    }
}
impl ViewSize<Size> for LockedView {
    fn size(&mut self, size: &Size) -> Size {
        *size
    }
}

fn locked_card_view<G: ViewGrid>(offset: Coord, depth: i32, grid: &mut G) {
    let border = Border {
        foreground_colour: grey24(128),
        ..Border::new()
    };
    Decorated::new(LockedView, border).view(
        &(CARD_SIZE - Coord::new(1, 1)).to_size().unwrap(),
        offset,
        depth,
        grid,
    );
}

fn empty_card_view<G: ViewGrid>(offset: Coord, depth: i32, grid: &mut G) {
    let view_cell = ViewCell::new()
        .with_character('░')
        .with_foreground(grey24(20));
    for coord in XThenYIter::new(CARD_SIZE.to_size().unwrap()) {
        grid.set_cell(offset + coord + Coord::new(1, 1), depth, view_cell);
    }
}

impl View<Card> for CardView {
    fn view<G: ViewGrid>(
        &mut self,
        card: &Card,
        offset: Coord,
        depth: i32,
        grid: &mut G,
    ) {
        let pager = Pager::new(
            &card.description,
            CARD_SIZE.to_size().unwrap(),
            Default::default(),
        );
        RichStringView::with_info(TextInfo::default().bold().underline()).view(
            &card.title,
            offset,
            depth + 1,
            grid,
        );
        PagerView.view(&pager, offset + Coord::new(0, 2), depth + 1, grid);
        for coord in XThenYIter::new(CARD_SIZE.to_size().unwrap()) {
            grid.set_cell(
                offset + coord,
                depth,
                ViewCell::new().with_background(card.background),
            );
        }
        let shadow_colour = card.background;
        let shadow_ch = '░';
        let shadow_bottom_ch = shadow_ch;
        let shadow_right_ch = shadow_ch;
        let shadow_bottom_right_ch = shadow_ch;
        for i in 0..(CARD_SIZE.x - 1) {
            let coord = Coord::new(i + 1, CARD_SIZE.y);
            grid.set_cell(
                offset + coord,
                depth,
                ViewCell::new()
                    .with_character(shadow_bottom_ch)
                    .with_foreground(shadow_colour),
            );
        }
        for i in 0..(CARD_SIZE.y - 1) {
            let coord = Coord::new(CARD_SIZE.x, i + 1);
            grid.set_cell(
                offset + coord,
                depth,
                ViewCell::new()
                    .with_character(shadow_right_ch)
                    .with_foreground(shadow_colour),
            );
        }
        grid.set_cell(
            offset + CARD_SIZE,
            depth,
            ViewCell::new()
                .with_character(shadow_bottom_right_ch)
                .with_foreground(shadow_colour),
        );
    }
}

pub struct DeathView;

impl View<Gws> for DeathView {
    fn view<G: ViewGrid>(&mut self, game: &Gws, offset: Coord, depth: i32, grid: &mut G) {
        DeathGameView.view(game, offset + GAME_OFFSET, depth, grid);
        DefaultRichTextView.view(
            // TODO avoid allocating on each frame
            &RichText::one_line(vec![
                (
                    "YOU DIED",
                    TextInfo::default()
                        .bold()
                        .foreground_colour(rgb24(255, 0, 0))
                        .background_colour(grey24(0)),
                ),
                (
                    " (press any key)",
                    TextInfo::default()
                        .bold()
                        .foreground_colour(grey24(255))
                        .background_colour(grey24(0)),
                ),
            ]),
            offset + TOP_TEXT_OFFSET,
            depth + 1,
            grid,
        );
        StatusView.view(game, offset + STATUS_OFFSET, depth, grid);
        CardAreaView.view(&test_cards(), offset + CARDS_OFFSET, depth, grid);
    }
}
