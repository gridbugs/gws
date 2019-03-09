use crate::ui::*;
use gws::*;
use prototty::*;
use rand::*;

const NORMAL_COLOUR: Rgb24 = Rgb24::new(100, 100, 150);
const SELECTED_COLOUR: Rgb24 = Rgb24::new(0, 120, 240);

fn instantiate_menu<T: Copy>(mut menu: Menu<T>) -> MenuInstance<T> {
    menu.normal_info = TextInfo {
        foreground_colour: Some(NORMAL_COLOUR),
        background_colour: None,
        bold: false,
        underline: false,
    };
    menu.selected_info = TextInfo {
        foreground_colour: Some(SELECTED_COLOUR),
        background_colour: None,
        bold: true,
        underline: false,
    };
    MenuInstance::new(menu).unwrap()
}
pub mod menu {
    use super::*;

    #[derive(Clone, Copy)]
    pub enum Choice {
        NewGame,
        Help,
        Quit,
    }

    pub fn create() -> MenuInstance<Choice> {
        instantiate_menu(Menu::smallest(vec![
            ("New Game", Choice::NewGame),
            ("Help", Choice::Help),
            ("Quit", Choice::Quit),
        ]))
    }
}

pub mod pause_menu {
    use super::*;

    #[derive(Clone, Copy)]
    pub enum Choice {
        Resume,
        NewGame,
        Help,
        Map,
        SaveAndQuit,
    }

    pub fn create() -> MenuInstance<Choice> {
        instantiate_menu(Menu::smallest(vec![
            ("Resume", Choice::Resume),
            ("Map", Choice::Map),
            ("Help", Choice::Help),
            ("New Game", Choice::NewGame),
            ("Save and Quit ", Choice::SaveAndQuit),
        ]))
    }
}
pub struct MenuAndTitle<'a, T: Copy> {
    pub menu: &'a MenuInstance<T>,
    pub title: &'a str,
}

impl<'a, T: Copy> MenuAndTitle<'a, T> {
    pub fn new(menu: &'a MenuInstance<T>, title: &'a str) -> Self {
        Self { menu, title }
    }
}

pub struct MenuAndTitleView {
    pub title_view: RichStringView,
    pub menu_view: DefaultMenuInstanceView,
}

impl MenuAndTitleView {
    pub fn new(colour: Rgb24) -> Self {
        Self {
            title_view: RichStringView::with_info(TextInfo {
                bold: true,
                underline: false,
                foreground_colour: Some(colour),
                background_colour: None,
            }),
            menu_view: DefaultMenuInstanceView::new(),
        }
    }
}

impl<'a, T: Copy> View<MenuAndTitle<'a, T>> for MenuAndTitleView {
    fn view<G: ViewGrid>(
        &mut self,
        &MenuAndTitle { menu, title }: &MenuAndTitle<'a, T>,
        offset: Coord,
        depth: i32,
        grid: &mut G,
    ) {
        self.title_view.view(title, offset, depth, grid);
        self.menu_view
            .view(menu, offset + Coord::new(0, 2), depth, grid);
    }
}

pub mod card_menu {
    use super::*;

    pub fn create(cards: &[Card], card_table: &CardTable) -> Option<MenuInstance<Card>> {
        let mut cards = cards.iter().collect::<Vec<_>>();
        cards.sort();
        let menu = Menu::smallest(
            cards
                .iter()
                .map(|&&card| {
                    let info = card_table.get(card);
                    let text = info.to_string();
                    (text, card)
                })
                .collect::<Vec<_>>(),
        );
        MenuInstance::new(menu)
    }
}

pub mod altar_menu {
    use super::*;

    fn upgrade_text(upgrade: CharacterUpgrade) -> &'static str {
        use CharacterUpgrade::*;
        match upgrade {
            Life => "Increase Max Life",
            Hand => "Increase Hand Size",
            Power => "Increase Max Power",
            Vision => "Increase Vision",
        }
    }

    pub type T = MenuInstance<(CharacterUpgrade, Card)>;

    pub fn create<R: Rng>(game: &Gws, card_table: &CardTable, rng: &mut R) -> T {
        let num_choices = 3;
        let choices = game
            .choose_upgrades(num_choices, rng)
            .cloned()
            .zip(game.choose_negative_cards(num_choices, rng).cloned());
        let menu = Menu::smallest(
            choices
                .map(|(upgrade, card)| {
                    let info = card_table.get(card);
                    let text = format!("{}, {}", upgrade_text(upgrade), info.to_string());
                    (text, (upgrade, card))
                })
                .collect::<Vec<_>>(),
        );
        MenuInstance::new(menu).unwrap()
    }
}

pub mod fountain_menu {
    use super::*;
    use rand::seq::SliceRandom;

    pub type T = MenuInstance<(Card, usize)>;

    const COUNTS: &'static [usize] = &[3, 4, 4, 4, 5, 5, 6];
    pub fn create<R: Rng>(game: &Gws, card_table: &CardTable, rng: &mut R) -> T {
        let num_choices = 3;
        let choices = game
            .choose_positive_cards(num_choices, rng)
            .cloned()
            .zip(COUNTS.choose_multiple(rng, num_choices).cloned());
        let menu = Menu::smallest(
            choices
                .map(|(card, count)| {
                    let info = card_table.get(card);
                    let text =
                        format!("{} x {}: {}", count, info.title, info.description);
                    (text, (card, count))
                })
                .collect::<Vec<_>>(),
        );
        MenuInstance::new(menu).unwrap()
    }
}
