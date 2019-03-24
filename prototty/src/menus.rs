use crate::ui::*;
use gws::*;
use prototty::*;

const NORMAL_COLOUR: Rgb24 = Rgb24::new(100, 100, 150);
const SELECTED_COLOUR: Rgb24 = Rgb24::new(0, 120, 240);

const SELECTED_VIEW: StringViewSingleLine =
    StringViewSingleLine::new(Style::new().with_foreground(SELECTED_COLOUR));
const NORMAL_VIEW: StringViewSingleLine =
    StringViewSingleLine::new(Style::new().with_foreground(NORMAL_COLOUR));

pub mod main {
    use super::*;

    #[derive(Clone, Copy, Debug)]
    pub enum Entry {
        NewGame,
        Help,
        Story,
        Quit,
    }

    pub fn choices() -> Vec<Entry> {
        vec![Entry::NewGame, Entry::Story, Entry::Help, Entry::Quit]
    }

    pub struct EntryView;

    impl MenuEntryView<Entry> for EntryView {
        fn normal<G: ViewGrid, R: ViewTransformRgb24>(
            &mut self,
            &choice: &Entry,
            context: ViewContext<R>,
            grid: &mut G,
        ) -> u32 {
            let string = match choice {
                Entry::NewGame => "  New Game",
                Entry::Help => "  Help",
                Entry::Story => "  Story",
                Entry::Quit => "  Quit",
            };
            NORMAL_VIEW
                .view_reporting_intended_size(string, context, grid)
                .width()
        }
        fn selected<G: ViewGrid, R: ViewTransformRgb24>(
            &mut self,
            &choice: &Entry,
            context: ViewContext<R>,
            grid: &mut G,
        ) -> u32 {
            let string = match choice {
                Entry::NewGame => "> New Game",
                Entry::Help => "> Help",
                Entry::Story => "> Story",
                Entry::Quit => "> Quit",
            };
            SELECTED_VIEW
                .view_reporting_intended_size(string, context, grid)
                .width()
        }
    }
}

pub mod pause {
    use super::*;

    #[derive(Clone, Copy, Debug)]
    pub enum Entry {
        Resume,
        NewGame,
        Help,
        Map,
        Story,
        SaveAndQuit,
    }

    pub fn choices() -> Vec<Entry> {
        vec![
            Entry::Resume,
            Entry::Map,
            Entry::Help,
            Entry::NewGame,
            Entry::Story,
            Entry::SaveAndQuit,
        ]
    }

    pub struct EntryView;

    impl MenuEntryView<Entry> for EntryView {
        fn normal<G: ViewGrid, R: ViewTransformRgb24>(
            &mut self,
            &choice: &Entry,
            context: ViewContext<R>,
            grid: &mut G,
        ) -> u32 {
            let string = match choice {
                Entry::Resume => "  Resume",
                Entry::NewGame => "  New Game",
                Entry::Help => "  Help",
                Entry::Map => "  Map",
                Entry::Story => "  Story",
                Entry::SaveAndQuit => "  Save and Quit",
            };
            StringViewSingleLine::new(Style::new().with_foreground(NORMAL_COLOUR))
                .view_reporting_intended_size(string, context, grid)
                .width()
        }
        fn selected<G: ViewGrid, R: ViewTransformRgb24>(
            &mut self,
            &choice: &Entry,
            context: ViewContext<R>,
            grid: &mut G,
        ) -> u32 {
            let string = match choice {
                Entry::Resume => "> Resume",
                Entry::NewGame => "> New Game",
                Entry::Help => "> Help",
                Entry::Map => "> Map",
                Entry::Story => "> Story",
                Entry::SaveAndQuit => "> Save and Quit",
            };
            StringViewSingleLine::new(Style::new().with_foreground(SELECTED_COLOUR))
                .view_reporting_intended_size(string, context, grid)
                .width()
        }
    }
}

pub mod card {
    use super::*;

    pub fn create(cards: &[Card]) -> Vec<Card> {
        let mut cards = cards.iter().cloned().collect::<Vec<_>>();
        cards.sort();
        cards
    }

    pub struct EntryView;

    impl MenuEntryLookupView<Card, CardTable> for EntryView {
        fn normal<G: ViewGrid, R: ViewTransformRgb24>(
            &mut self,
            &card: &Card,
            card_table: &CardTable,
            context: ViewContext<R>,
            grid: &mut G,
        ) -> u32 {
            let string = format!("  {}", card_table.get(card).to_string());
            NORMAL_VIEW
                .view_reporting_intended_size(&string, context, grid)
                .width()
        }
        fn selected<G: ViewGrid, R: ViewTransformRgb24>(
            &mut self,
            &card: &Card,
            card_table: &CardTable,
            context: ViewContext<R>,
            grid: &mut G,
        ) -> u32 {
            let string = format!("> {}", card_table.get(card).to_string());
            SELECTED_VIEW
                .view_reporting_intended_size(&string, context, grid)
                .width()
        }
    }
}

pub mod altar {
    use super::*;

    pub type Entry = (CharacterUpgrade, Card);
    pub fn choices(id: EntityId, game: &Gws) -> Vec<Entry> {
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
        choices.collect()
    }

    fn upgrade_text(upgrade: CharacterUpgrade) -> &'static str {
        use CharacterUpgrade::*;
        match upgrade {
            Life => "Increase Max Life",
            Hand => "Increase Hand Size",
            Power => "Increase Max Power",
            Vision => "Increase Vision",
        }
    }

    pub struct EntryView;

    impl MenuEntryLookupView<Entry, CardTable> for EntryView {
        fn normal<G: ViewGrid, R: ViewTransformRgb24>(
            &mut self,
            &(character_upgrade, card): &Entry,
            card_table: &CardTable,
            context: ViewContext<R>,
            grid: &mut G,
        ) -> u32 {
            let text = format!(
                "  {}, {}",
                upgrade_text(character_upgrade),
                card_table.get(card).to_string()
            );
            NORMAL_VIEW
                .view_reporting_intended_size(&text, context, grid)
                .width()
        }
        fn selected<G: ViewGrid, R: ViewTransformRgb24>(
            &mut self,
            &(character_upgrade, card): &Entry,
            card_table: &CardTable,
            context: ViewContext<R>,
            grid: &mut G,
        ) -> u32 {
            let text = format!(
                "> {}, {}",
                upgrade_text(character_upgrade),
                card_table.get(card).to_string()
            );
            SELECTED_VIEW
                .view_reporting_intended_size(&text, context, grid)
                .width()
        }
    }
}

pub mod fountain {
    use super::*;

    pub type Entry = (Card, usize);

    pub fn choices(id: EntityId, game: &Gws) -> Vec<Entry> {
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
        choices.collect()
    }

    pub struct EntryView;

    impl MenuEntryLookupView<Entry, CardTable> for EntryView {
        fn normal<G: ViewGrid, R: ViewTransformRgb24>(
            &mut self,
            &(card, count): &Entry,
            card_table: &CardTable,
            context: ViewContext<R>,
            grid: &mut G,
        ) -> u32 {
            let text = format!("  {} x {}", count, card_table.get(card).to_string());
            NORMAL_VIEW
                .view_reporting_intended_size(&text, context, grid)
                .width()
        }
        fn selected<G: ViewGrid, R: ViewTransformRgb24>(
            &mut self,
            &(card, count): &Entry,
            card_table: &CardTable,
            context: ViewContext<R>,
            grid: &mut G,
        ) -> u32 {
            let text = format!("> {} x {}", count, card_table.get(card).to_string());
            SELECTED_VIEW
                .view_reporting_intended_size(&text, context, grid)
                .width()
        }
    }
}
