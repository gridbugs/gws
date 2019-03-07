use crate::ui::*;
use gws::*;
use prototty::*;

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
    pub fn new() -> Self {
        Self {
            title_view: RichStringView::with_info(TextInfo {
                bold: true,
                underline: false,
                foreground_colour: Some(SELECTED_COLOUR),
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
                    let text = format!("{}: {}", info.title, info.description);
                    (text, card)
                })
                .collect::<Vec<_>>(),
        );
        MenuInstance::new(menu)
    }
}
