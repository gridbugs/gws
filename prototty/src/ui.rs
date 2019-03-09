use crate::game_view::*;
use crate::*;
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

const GAME_SIZE: Coord = Coord::new(60, 40);

const MESSAGE_OFFSET: Coord = Coord {
    x: STATUS_OFFSET.x,
    y: GAME_OFFSET.y + GAME_SIZE.y + 1,
};

const MESSAGE_HEIGHT: i32 = 1;

const CARDS_OFFSET: Coord = Coord {
    x: STATUS_OFFSET.x,
    y: MESSAGE_OFFSET.y + MESSAGE_HEIGHT + 1,
};
const CARD_SIZE: Coord = Coord::new(8, 10);
const CARD_PADDING_X: i32 = 1;

const MAX_NUM_CARDS: usize = 8;

struct StatusView;

pub struct UiData<'a> {
    pub game: &'a Gws,
    pub message: Option<&'a str>,
    pub card_table: &'a CardTable,
    pub card_selection: Option<&'a CardInSlot>,
}

impl<'a> View<UiData<'a>> for StatusView {
    fn view<G: ViewGrid>(
        &mut self,
        ui_data: &UiData<'a>,
        offset: Coord,
        depth: i32,
        grid: &mut G,
    ) {
        let to_render = ui_data.game.to_render();
        let player_hit_points = to_render.player.hit_points().unwrap();
        let health_colour = rgb24(200, 0, 0);
        let time_colour = rgb24(0, 200, 0);
        let deck_colour = rgb24(180, 180, 0);
        let spent_colour = rgb24(100, 70, 180);
        let waste_colour = rgb24(100, 120, 20);
        let burnt_colour = rgb24(150, 100, 40);
        let draw_countdown = ui_data.game.draw_countdown();
        let mut offset = offset;
        StringView.view("Life:", offset, depth, grid);
        RichStringView::with_info(
            TextInfo::default().bold().foreground_colour(health_colour),
        )
        .view(
            &format!("{}/{}", player_hit_points.current, player_hit_points.max),
            offset + Coord::new(7, 0),
            depth,
            grid,
        );
        offset += Coord::new(0, 2);
        StringView.view("Power:", offset, depth, grid);
        RichStringView::with_info(
            TextInfo::default().bold().foreground_colour(time_colour),
        )
        .view(
            &format!("{}/{}", draw_countdown.current, draw_countdown.max),
            offset + Coord::new(7, 0),
            depth,
            grid,
        );
        offset += Coord::new(0, 2);
        StringView.view("Deck:", offset, depth, grid);
        RichStringView::with_info(
            TextInfo::default().bold().foreground_colour(deck_colour),
        )
        .view(
            &format!("{}", ui_data.game.deck().len()),
            offset + Coord::new(7, 0),
            depth,
            grid,
        );
        offset += Coord::new(0, 2);
        StringView.view("Spent:", offset, depth, grid);
        RichStringView::with_info(
            TextInfo::default().bold().foreground_colour(spent_colour),
        )
        .view(
            &format!("{}", ui_data.game.spent().len()),
            offset + Coord::new(7, 0),
            depth,
            grid,
        );
        offset += Coord::new(0, 2);
        StringView.view("Waste:", offset, depth, grid);
        RichStringView::with_info(
            TextInfo::default().bold().foreground_colour(waste_colour),
        )
        .view(
            &format!("{}", ui_data.game.waste().len()),
            offset + Coord::new(7, 0),
            depth,
            grid,
        );
        offset += Coord::new(0, 2);
        StringView.view("Burnt:", offset, depth, grid);
        RichStringView::with_info(
            TextInfo::default().bold().foreground_colour(burnt_colour),
        )
        .view(
            &format!("{}", ui_data.game.burnt().len()),
            offset + Coord::new(7, 0),
            depth,
            grid,
        );
    }
}

impl<'a, V: View<Gws>> View<UiData<'a>> for UiView<V> {
    fn view<G: ViewGrid>(
        &mut self,
        ui_data: &UiData<'a>,
        offset: Coord,
        depth: i32,
        grid: &mut G,
    ) {
        self.0.view(ui_data.game, offset + GAME_OFFSET, depth, grid);
        if let Some(card_selection) = ui_data.card_selection {
            match card_selection.choice {
                CardParamChoice::Confirm | CardParamChoice::Direction => (),
                CardParamChoice::Coord(coord) => {
                    if coord.x >= 0
                        && coord.y >= 0
                        && coord.x < GAME_SIZE.x
                        && coord.y < GAME_SIZE.y
                    {
                        grid.set_cell(
                            offset + GAME_OFFSET + coord,
                            depth + 1,
                            ViewCell::new()
                                .with_character('Х')
                                .with_bold(true)
                                .with_foreground(rgb24(0, 255, 255)),
                        );
                    }
                }
            }
        }
        StatusView.view(ui_data, offset + STATUS_OFFSET, depth, grid);
        CardAreaView.view(
            &(
                ui_data.game.hand(),
                ui_data.card_table,
                ui_data.card_selection.map(|cs| cs.slot),
                *ui_data.game.draw_countdown(),
            ),
            offset + CARDS_OFFSET,
            depth,
            grid,
        );
        if let Some(message) = ui_data.message {
            StringView.view(message, offset + MESSAGE_OFFSET, depth, grid);
        }
    }
}

struct CardView;
struct CardAreaView;

pub struct CardInfo {
    pub card: Card,
    pub title: String,
    pub description: String,
    pub description_pager: Pager,
    pub background: Rgb24,
}

impl CardInfo {
    fn new(card: Card, title: String, description: String, background: Rgb24) -> Self {
        let description_pager = Pager::new(
            &description,
            CARD_SIZE.to_size().unwrap(),
            Default::default(),
        );
        Self {
            card,
            title,
            description,
            description_pager,
            background,
        }
    }
}

pub struct CardTable {
    bump: CardInfo,
    blink: CardInfo,
    heal: CardInfo,
    spark: CardInfo,
    clog: CardInfo,
    parasite: CardInfo,
    drain: CardInfo,
}

impl CardTable {
    pub fn new() -> Self {
        Self {
            bump: CardInfo::new(
                Card::Bump,
                "Bump".to_string(),
                "Attack adjacent square for 1 damage".to_string(),
                rgb24(20, 0, 0),
            ),
            blink: CardInfo::new(
                Card::Blink,
                "Blink".to_string(),
                "Teleport to visible square up to 8 away".to_string(),
                rgb24(0, 20, 0),
            ),
            heal: CardInfo::new(
                Card::Heal,
                "Heal".to_string(),
                "Recover 1 hit point".to_string(),
                rgb24(0, 20, 0),
            ),
            spark: CardInfo::new(
                Card::Spark,
                "Spark".to_string(),
                "Shoot a spark dealing 1 damage with 12 range.".to_string(),
                rgb24(20, 0, 0),
            ),
            clog: CardInfo::new(
                Card::Clog,
                "Clog".to_string(),
                "Has no effect.".to_string(),
                rgb24(20, 20, 20),
            ),
            parasite: CardInfo::new(
                Card::Parasite,
                "Parasite".to_string(),
                "Take 2 damage. Burn this card.".to_string(),
                rgb24(20, 20, 20),
            ),
            drain: CardInfo::new(
                Card::Drain,
                "Drain".to_string(),
                "Burn this card.".to_string(),
                rgb24(20, 20, 20),
            ),
        }
    }
    pub fn get(&self, card: Card) -> &CardInfo {
        match card {
            Card::Bump => &self.bump,
            Card::Blink => &self.blink,
            Card::Heal => &self.heal,
            Card::Spark => &self.spark,
            Card::Clog => &self.clog,
            Card::Parasite => &self.parasite,
            Card::Drain => &self.drain,
        }
    }
}

impl<'a>
    View<(
        &'a [Option<Card>],
        &'a CardTable,
        Option<usize>,
        DrawCountdown,
    )> for CardAreaView
{
    fn view<G: ViewGrid>(
        &mut self,
        &(cards, card_table, selected_slot, draw_countdown): &(
            &'a [Option<Card>],
            &'a CardTable,
            Option<usize>,
            DrawCountdown,
        ),
        offset: Coord,
        depth: i32,
        grid: &mut G,
    ) {
        for i in 0..MAX_NUM_CARDS {
            let offset_x = i as i32 * (CARD_SIZE.x + CARD_PADDING_X);
            StringView.view(
                &format!("{}.", i + 1),
                offset + Coord::new(offset_x + 4, 0),
                depth,
                grid,
            );
            let coord = offset + Coord::new(offset_x, 1);
            if let Some(maybe_card) = cards.get(i) {
                if let Some(card) = maybe_card.as_ref() {
                    let selected =
                        selected_slot.map(|s| s == i as usize).unwrap_or(false);
                    CardView.view(
                        &(card_table.get(*card), selected, draw_countdown),
                        coord,
                        depth,
                        grid,
                    );
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

impl<'a> View<(&'a CardInfo, bool, DrawCountdown)> for CardView {
    fn view<G: ViewGrid>(
        &mut self,
        &(card_info, selected, draw_countdown): &(&'a CardInfo, bool, DrawCountdown),
        offset: Coord,
        depth: i32,
        grid: &mut G,
    ) {
        let selected_offset = if selected {
            Coord::new(0, 0)
        } else {
            Coord::new(1, 1)
        };
        RichStringView::with_info(TextInfo::default().bold().underline()).view(
            &card_info.title,
            offset + selected_offset,
            depth + 1,
            grid,
        );
        PagerView.view(
            &card_info.description_pager,
            offset + selected_offset + Coord::new(0, 2),
            depth + 1,
            grid,
        );
        let energy_cost = card_info.card.cost();
        let energy_colour = if draw_countdown.current < energy_cost {
            rgb24(255, 0, 0)
        } else {
            rgb24(255, 255, 255)
        };
        DefaultRichTextView.view(
            &RichText::one_line(vec![
                ("Cost: ".to_string(), Default::default()),
                (
                    format!("{}", energy_cost),
                    TextInfo::default().foreground_colour(energy_colour).bold(),
                ),
            ]),
            offset + selected_offset + Coord::new(0, 9),
            depth + 1,
            grid,
        );
        let background = if selected {
            card_info.background.saturating_scalar_mul_div(2, 1)
        } else {
            card_info.background
        };
        for coord in XThenYIter::new(CARD_SIZE.to_size().unwrap()) {
            grid.set_cell(
                offset + selected_offset + coord,
                depth,
                ViewCell::new().with_background(background),
            );
        }
        if selected {
            let shadow_colour = background;
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
}

pub struct DeathView;

impl<'a> View<UiData<'a>> for DeathView {
    fn view<G: ViewGrid>(
        &mut self,
        ui_data: &UiData<'a>,
        offset: Coord,
        depth: i32,
        grid: &mut G,
    ) {
        DeathGameView.view(ui_data.game, offset + GAME_OFFSET, depth, grid);
        StatusView.view(ui_data, offset + STATUS_OFFSET, depth, grid);
        StringView.view(
            "You died. Press any key...",
            offset + MESSAGE_OFFSET,
            depth,
            grid,
        );
        CardAreaView.view(
            &(
                ui_data.game.hand(),
                ui_data.card_table,
                None,
                *ui_data.game.draw_countdown(),
            ),
            offset + CARDS_OFFSET,
            depth,
            grid,
        );
    }
}
