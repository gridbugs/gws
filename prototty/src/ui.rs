use crate::game_view::*;
use grid_2d::coord_system::XThenYIter;
use gws::*;
use prototty::*;

pub struct UiView<V: View<Gws>>(pub V);

const GAME_OFFSET: Coord = Coord::new(1, 1);

const TOP_TEXT_OFFSET: Coord = Coord::new(1, 1);
const GAME_SIZE: Coord = Coord::new(60, 40);
const CARDS_OFFSET: Coord = Coord {
    x: GAME_OFFSET.x,
    y: GAME_OFFSET.y + GAME_SIZE.y + 1,
};
const CARD_SIZE: Coord = Coord::new(8, 10);
const CARD_PADDING_X: i32 = 1;

fn test_cards() -> Vec<Card> {
    vec![
        Card {
            title: "Bump".to_string(),
            description: "Attack adjacent square for 1 damage".to_string(),
            background: rgb24(20, 0, 0),
        },
        Card {
            title: "Blink".to_string(),
            description: "Teleport to selected square".to_string(),
            background: rgb24(0, 20, 0),
        },
        Card {
            title: "Bump".to_string(),
            description: "Attack adjacent square for 1 damage".to_string(),
            background: rgb24(20, 0, 0),
        },
        Card {
            title: "Blink".to_string(),
            description: "Teleport to selected square".to_string(),
            background: rgb24(0, 20, 0),
        },
        Card {
            title: "Blink".to_string(),
            description: "Teleport to selected square".to_string(),
            background: rgb24(0, 20, 0),
        },
        Card {
            title: "Blink".to_string(),
            description: "Teleport to selected square".to_string(),
            background: rgb24(0, 20, 0),
        },
        Card {
            title: "Blink".to_string(),
            description: "Teleport to selected square".to_string(),
            background: rgb24(0, 20, 0),
        },
        Card {
            title: "Blink".to_string(),
            description: "Teleport to selected square".to_string(),
            background: rgb24(0, 20, 0),
        },
    ]
}

impl<V: View<Gws>> View<Gws> for UiView<V> {
    fn view<G: ViewGrid>(&mut self, game: &Gws, offset: Coord, depth: i32, grid: &mut G) {
        self.0.view(game, offset + GAME_OFFSET, depth, grid);
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

impl View<[Card]> for CardAreaView {
    fn view<G: ViewGrid>(
        &mut self,
        cards: &[Card],
        offset: Coord,
        depth: i32,
        grid: &mut G,
    ) {
        for (i, card) in cards.iter().enumerate() {
            let offset_x = i as i32 * (CARD_SIZE.x + CARD_PADDING_X);
            StringView.view(
                &format!("{}.", i + 1),
                offset + Coord::new(offset_x + 3, 0),
                depth,
                grid,
            );
            CardView.view(card, offset + Coord::new(offset_x, 1), depth, grid);
        }
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
                        .foreground_colour(rgb24(128, 0, 0))
                        .background_colour(grey24(0)),
                ),
                (
                    " (press any key)",
                    TextInfo::default()
                        .bold()
                        .foreground_colour(grey24(128))
                        .background_colour(grey24(0)),
                ),
            ]),
            offset + TOP_TEXT_OFFSET,
            depth + 1,
            grid,
        );
        CardAreaView.view(&test_cards(), offset + CARDS_OFFSET, depth, grid);
    }
}
