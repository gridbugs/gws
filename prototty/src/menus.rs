use crate::ui::*;
use gws::*;
use prototty::*;
use rand::*;

const NORMAL_COLOUR: Rgb24 = Rgb24::new(100, 100, 150);
const SELECTED_COLOUR: Rgb24 = Rgb24::new(0, 120, 240);

fn instantiate_menu<T: Copy>(mut menu: Vec<T>) -> MenuInstance<T> {
    MenuInstance::new(menu).unwrap()
}
pub mod menu {
    use super::*;

    #[derive(Clone, Copy, Debug)]
    pub enum Choice {
        NewGame,
        Help,
        Story,
        Quit,
    }

    pub struct EntryView;

    impl MenuEntryView<Choice> for EntryView {
        fn normal<G: ViewGrid, R: ViewTransformRgb24>(
            &mut self,
            &choice: &Choice,
            context: ViewContext<R>,
            grid: &mut G,
        ) -> Size {
            let string = match choice {
                Choice::NewGame => "  New Game",
                Choice::Help => "  Help",
                Choice::Story => "  Story",
                Choice::Quit => "  Quit",
            };
            StringViewSingleLine::new(Style::new().with_foreground(NORMAL_COLOUR))
                .view_reporting_intended_size(string, context, grid)
        }
        fn selected<G: ViewGrid, R: ViewTransformRgb24>(
            &mut self,
            &choice: &Choice,
            context: ViewContext<R>,
            grid: &mut G,
        ) -> Size {
            let string = match choice {
                Choice::NewGame => "> New Game",
                Choice::Help => "> Help",
                Choice::Story => "> Story",
                Choice::Quit => "> Quit",
            };
            StringViewSingleLine::new(Style::new().with_foreground(SELECTED_COLOUR))
                .view_reporting_intended_size(string, context, grid)
        }
    }

    pub fn create() -> MenuInstance<Choice> {
        instantiate_menu(vec![
            Choice::NewGame,
            Choice::Story,
            Choice::Help,
            Choice::Quit,
        ])
    }
}

pub mod pause_menu {
    use super::*;

    #[derive(Clone, Copy, Debug)]
    pub enum Choice {
        Resume,
        NewGame,
        Help,
        Map,
        Story,
        SaveAndQuit,
    }

    pub struct EntryView;

    impl MenuEntryView<Choice> for EntryView {
        fn normal<G: ViewGrid, R: ViewTransformRgb24>(
            &mut self,
            &choice: &Choice,
            context: ViewContext<R>,
            grid: &mut G,
        ) -> Size {
            let string = match choice {
                Choice::Resume => "  Resume",
                Choice::NewGame => "  New Game",
                Choice::Help => "  Help",
                Choice::Map => "  Map",
                Choice::Story => "  Story",
                Choice::SaveAndQuit => "  Save and Quit",
            };
            StringViewSingleLine::new(Style::new().with_foreground(NORMAL_COLOUR))
                .view_reporting_intended_size(string, context, grid)
        }
        fn selected<G: ViewGrid, R: ViewTransformRgb24>(
            &mut self,
            &choice: &Choice,
            context: ViewContext<R>,
            grid: &mut G,
        ) -> Size {
            let string = match choice {
                Choice::Resume => "> Resume",
                Choice::NewGame => "> New Game",
                Choice::Help => "> Help",
                Choice::Map => "> Map",
                Choice::Story => "> Story",
                Choice::SaveAndQuit => "> Save and Quit",
            };
            StringViewSingleLine::new(Style::new().with_foreground(SELECTED_COLOUR))
                .view_reporting_intended_size(string, context, grid)
        }
    }

    pub fn create() -> MenuInstance<Choice> {
        instantiate_menu(vec![
            Choice::Resume,
            Choice::Map,
            Choice::Help,
            Choice::NewGame,
            Choice::Story,
            Choice::SaveAndQuit,
        ])
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

pub struct MenuAndTitleView<E> {
    pub title_view: StringViewSingleLine,
    pub menu_view: MenuInstanceView<E>,
}

impl<E> MenuAndTitleView<E> {
    pub fn new(colour: Rgb24, entry_view: E) -> Self {
        Self {
            title_view: StringViewSingleLine::new(
                Style::new().with_bold(true).with_foreground(colour),
            ),
            menu_view: MenuInstanceView::new(entry_view),
        }
    }
}

pub struct DummyEntryView;
impl<T> MenuEntryView<T> for DummyEntryView {
    fn normal<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        entry: &T,
        context: ViewContext<R>,
        grid: &mut G,
    ) -> Size {
        Size::new(0, 0)
    }
    fn selected<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        entry: &T,
        context: ViewContext<R>,
        grid: &mut G,
    ) -> Size {
        Size::new(0, 0)
    }
}

impl<'a, T: Copy, E: MenuEntryView<T>> View<MenuAndTitle<'a, T>> for MenuAndTitleView<E> {
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        MenuAndTitle { menu, title }: MenuAndTitle<'a, T>,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        self.title_view.view(title, context, grid);
        self.menu_view
            .view(menu, context.add_offset(Coord::new(0, 2)), grid);
    }
}

pub mod card_menu {
    use super::*;

    pub fn create(cards: &[Card], card_table: &CardTable) -> Option<MenuInstance<Card>> {
        unimplemented!()
        /*
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
        MenuInstance::new(menu).ok() */
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

    pub fn create<R: Rng>(
        id: EntityId,
        game: &Gws,
        card_table: &CardTable,
        _rng: &mut R,
    ) -> T {
        unimplemented!()
        /*
        let upgrade = game
            .to_render()
            .world
            .entities()
            .get(&id)
            .unwrap()
            .upgrade()
            .unwrap();
        let choices = upgrade
            .character_upgrades
            .iter()
            .cloned()
            .zip(upgrade.negative_cards.iter().cloned());
        let menu = Menu::smallest(
            choices
                .map(|(upgrade, card)| {
                    let info = card_table.get(card);
                    let text = format!("{}, {}", upgrade_text(upgrade), info.to_string());
                    (text, (upgrade, card))
                })
                .collect::<Vec<_>>(),
        );
        MenuInstance::new(menu).unwrap() */
    }
}

pub mod fountain_menu {
    use super::*;

    pub type Choice = (Card, usize);
    pub struct EntryView;
    impl MenuEntryView<Choice> for EntryView {
        fn normal<G: ViewGrid, R: ViewTransformRgb24>(
            &mut self,
            &(card, count): &Choice,
            context: ViewContext<R>,
            grid: &mut G,
        ) -> Size {
            Size::new(0, 0)
        }
        fn selected<G: ViewGrid, R: ViewTransformRgb24>(
            &mut self,
            &(card, count): &Choice,
            context: ViewContext<R>,
            grid: &mut G,
        ) -> Size {
            Size::new(0, 0)
        }
    }
    pub type T = MenuInstance<Choice>;

    pub fn create<R: Rng>(
        id: EntityId,
        game: &Gws,
        card_table: &CardTable,
        _rng: &mut R,
    ) -> T {
        unimplemented!()
        /*
        let upgrade = game
            .to_render()
            .world
            .entities()
            .get(&id)
            .unwrap()
            .upgrade()
            .unwrap();
        let choices = upgrade
            .positive_cards
            .iter()
            .cloned()
            .zip(upgrade.counts.iter().cloned());
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
        */
    }
}
