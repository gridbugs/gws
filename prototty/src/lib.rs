extern crate direction;
extern crate grid_2d;
extern crate gws;
extern crate prototty;
extern crate rand;
extern crate rand_isaac;
#[macro_use]
extern crate serde;

pub mod frontend;
mod game_view;
mod map_view;
mod menus;
mod ui;

use direction::*;
use game_view::GameView;
use map_view::MapView;
use menus::*;
use prototty::*;
use rand::{FromEntropy, Rng, SeedableRng};
use rand_isaac::IsaacRng;
use std::marker::PhantomData;
use std::time::Duration;
use ui::*;

const TITLE: &'static str = "Get Well Soon";
const AUTO_SAVE_PERIOD: Duration = Duration::from_millis(5000);

pub const APP_SIZE: Size = Size::new_u16(74, 58);
const TITLE_COLOUR: Rgb24 = Rgb24::new(0, 120, 240);

pub struct AppView {
    menu_and_title_view: MenuAndTitleView,
    flame_view: MenuAndTitleView,
    altar_view: MenuAndTitleView,
    fountain_view: MenuAndTitleView,
}

pub enum Tick {
    Quit,
    GameInitialisedWithSeed(u64),
    AutoSave,
}

pub enum InitStatus {
    NoSaveFound,
    LoadedSaveWithSeed(u64),
}

use frontend::Frontend;

const SAVE_KEY: &'static str = "save";

#[derive(Serialize, Deserialize)]
struct RngWithSeed {
    seed: u64,
    rng: IsaacRng,
}

#[derive(Serialize, Deserialize)]
struct GameState {
    rng_with_seed: RngWithSeed,
    all_inputs: Vec<gws::Input>,
    game: gws::Gws,
}

impl GameState {
    fn new(
        between_levels: Option<gws::BetweenLevels>,
        mut rng_with_seed: RngWithSeed,
        debug_terrain_string: Option<&str>,
    ) -> Self {
        let game =
            gws::Gws::new(between_levels, &mut rng_with_seed.rng, debug_terrain_string);
        Self {
            rng_with_seed,
            all_inputs: Vec::new(),
            game,
        }
    }
}

enum AppState {
    Game,
    Menu,
    Map { opened_from_game: bool },
    Help { opened_from_game: bool },
    BetweenLevels(Option<gws::BetweenLevels>),
    Death,
    CardMenu,
    AltarMenu,
    FountainMenu,
    ListDeck,
    ListSpent,
    ListWaste,
    ListBurnt,
    ViewCursor,
}

#[derive(Debug, Clone, Copy)]
pub enum FirstRngSeed {
    Seed(u64),
    Random,
}

struct RngSource {
    next_seed: u64,
    rng: IsaacRng,
}

impl RngSource {
    fn new(first_rng_seed: FirstRngSeed) -> Self {
        let mut rng = IsaacRng::from_entropy();
        let next_seed = match first_rng_seed {
            FirstRngSeed::Seed(seed) => seed,
            FirstRngSeed::Random => rng.gen(),
        };
        Self { next_seed, rng }
    }
    fn next(&mut self) -> RngWithSeed {
        let seed = self.next_seed;
        self.next_seed = self.rng.gen();
        let rng = IsaacRng::seed_from_u64(seed);
        RngWithSeed { seed, rng }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum CardParamChoice {
    Coord(Coord),
    Direction,
    Confirm,
}

pub struct CardInSlot {
    slot: usize,
    choice: CardParamChoice,
}

pub struct App<F: Frontend, S: Storage> {
    frontend: PhantomData<F>,
    storage: S,
    app_state: AppState,
    game_state: Option<GameState>,
    rng_source: RngSource,
    menu: MenuInstance<menu::Choice>,
    pause_menu: MenuInstance<pause_menu::Choice>,
    time_until_next_auto_save: Duration,
    help_pager: Pager,
    debug_terrain_string: Option<String>,
    message: Option<String>,
    card_table: CardTable,
    card_selection: Option<CardInSlot>,
    card_menu_title: String,
    card_menu: Option<MenuInstance<gws::Card>>,
    altar_menu: Option<menus::altar_menu::T>,
    fountain_menu: Option<menus::fountain_menu::T>,
    interactive: Option<gws::Interactive>,
    view_cursor: Option<Coord>,
}

fn list_cards<G>(
    cards: &[gws::Card],
    card_table: &CardTable,
    title: &str,
    offset: Coord,
    depth: i32,
    grid: &mut G,
) where
    G: ViewGrid,
{
    let mut cards = cards.iter().cloned().collect::<Vec<_>>();
    cards.sort();
    StringView.view(title, offset + Coord::new(1, 1), depth, grid);
    for (i, &card) in cards.iter().enumerate() {
        let info = card_table.get(card);
        StringView.view(
            &format!("{}: {}", info.title, info.description),
            offset + Coord::new(1, i as i32 + 3),
            depth,
            grid,
        );
    }
}

impl<F: Frontend, S: Storage> View<App<F, S>> for AppView {
    fn view<G>(&mut self, app: &App<F, S>, offset: Coord, depth: i32, grid: &mut G)
    where
        G: ViewGrid,
    {
        match app.app_state {
            AppState::ViewCursor => {
                if let Some(game_state) = app.game_state.as_ref() {
                    UiView(GameView).view(
                        &UiData {
                            game: &game_state.game,
                            message: app.message.as_ref().map(String::as_str),
                            card_table: &app.card_table,
                            card_selection: app.card_selection.as_ref(),
                            view_cursor: app.view_cursor.as_ref(),
                        },
                        offset,
                        depth,
                        grid,
                    );
                }
            }
            AppState::ListDeck => list_cards(
                app.game_state.as_ref().unwrap().game.deck(),
                &app.card_table,
                "Deck",
                offset,
                depth,
                grid,
            ),
            AppState::ListSpent => list_cards(
                app.game_state.as_ref().unwrap().game.spent(),
                &app.card_table,
                "Spent",
                offset,
                depth,
                grid,
            ),
            AppState::ListWaste => list_cards(
                app.game_state.as_ref().unwrap().game.waste(),
                &app.card_table,
                "Waste",
                offset,
                depth,
                grid,
            ),
            AppState::ListBurnt => list_cards(
                app.game_state.as_ref().unwrap().game.burnt(),
                &app.card_table,
                "Burnt",
                offset,
                depth,
                grid,
            ),
            AppState::AltarMenu => {
                self.altar_view.view(
                    &MenuAndTitle::new(
                        app.altar_menu.as_ref().unwrap(),
                        "Cursed Altar: Better yourself, but shuffle a cursed card into your deck.",
                    ),
                    offset + Coord::new(1, 1),
                    depth,
                    grid,
                );
            }
            AppState::FountainMenu => {
                self.fountain_view.view(
                    &MenuAndTitle::new(
                        app.fountain_menu.as_ref().unwrap(),
                        "Bountiful Fountain: Shuffle cards into your deck.",
                    ),
                    offset + Coord::new(1, 1),
                    depth,
                    grid,
                );
            }
            AppState::CardMenu => {
                self.flame_view.view(
                    &MenuAndTitle::new(
                        app.card_menu.as_ref().unwrap(),
                        app.card_menu_title.as_str(),
                    ),
                    offset + Coord::new(1, 1),
                    depth,
                    grid,
                );
            }
            AppState::Menu => {
                if app.game_state.is_some() {
                    self.menu_and_title_view.view(
                        &MenuAndTitle::new(&app.pause_menu, TITLE),
                        offset + Coord::new(1, 1),
                        depth,
                        grid,
                    );
                } else {
                    self.menu_and_title_view.view(
                        &MenuAndTitle::new(&app.menu, TITLE),
                        offset + Coord::new(1, 1),
                        depth,
                        grid,
                    );
                }
            }
            AppState::Game => {
                if let Some(game_state) = app.game_state.as_ref() {
                    UiView(GameView).view(
                        &UiData {
                            game: &game_state.game,
                            message: app.message.as_ref().map(String::as_str),
                            card_table: &app.card_table,
                            card_selection: app.card_selection.as_ref(),
                            view_cursor: None,
                        },
                        offset,
                        depth,
                        grid,
                    );
                }
            }
            AppState::Map { .. } => {
                if let Some(game_state) = app.game_state.as_ref() {
                    UiView(MapView).view(
                        &UiData {
                            game: &game_state.game,
                            message: app.message.as_ref().map(String::as_str),
                            card_table: &app.card_table,
                            card_selection: None,
                            view_cursor: None,
                        },
                        offset,
                        depth,
                        grid,
                    );
                }
            }
            AppState::Help { .. } => {
                PagerView.view(&app.help_pager, offset, depth, grid);
            }
            AppState::BetweenLevels(_) => {
                StringView.view(
                    "Generating level...",
                    offset + Coord::new(1, 1),
                    depth,
                    grid,
                );
            }
            AppState::Death => {
                if let Some(game_state) = app.game_state.as_ref() {
                    DeathView.view(
                        &UiData {
                            game: &game_state.game,
                            message: app.message.as_ref().map(String::as_str),
                            card_table: &app.card_table,
                            card_selection: None,
                            view_cursor: None,
                        },
                        offset,
                        depth,
                        grid,
                    );
                }
            }
        }
    }
}

const HELP_INPUT0: ProtottyInput = ProtottyInput::Char('h');
const HELP_INPUT1: ProtottyInput = ProtottyInput::Function(1);
const MAP_INPUT0: ProtottyInput = ProtottyInput::Char('m');
const MAP_INPUT1: ProtottyInput = ProtottyInput::Function(2);

impl<F: Frontend, S: Storage> App<F, S> {
    pub fn new(
        frontend: F,
        storage: S,
        first_rng_seed: FirstRngSeed,
        debug_terrain_string: Option<String>,
    ) -> (Self, InitStatus) {
        let _ = frontend;
        let (init_status, game_state) = match storage.load::<_, GameState>(SAVE_KEY) {
            Ok(game_state) => (
                InitStatus::LoadedSaveWithSeed(game_state.rng_with_seed.seed),
                Some(game_state),
            ),
            Err(_) => (InitStatus::NoSaveFound, None),
        };
        let rng_source = RngSource::new(first_rng_seed);
        let menu = menu::create();
        let pause_menu = pause_menu::create();
        let app = Self {
            frontend: PhantomData,
            storage,
            app_state: AppState::Menu,
            game_state,
            rng_source,
            menu,
            pause_menu,
            time_until_next_auto_save: AUTO_SAVE_PERIOD,
            help_pager: Pager::new(
                include_str!("help.txt"),
                // TODO in glutin the right line of cells is missing
                APP_SIZE - Size::new(1, 0),
                Default::default(),
            ),
            debug_terrain_string,
            message: None,
            card_table: CardTable::new(),
            card_selection: None,
            card_menu_title: "".to_string(),
            card_menu: None,
            altar_menu: None,
            fountain_menu: None,
            interactive: None,
            view_cursor: None,
        };
        (app, init_status)
    }
    pub fn save(&mut self) {
        if let Some(game_state) = self.game_state.as_ref() {
            self.storage
                .store(SAVE_KEY, &game_state)
                .expect("Failed to save game");
        } else {
            self.delete_save();
        }
    }
    pub fn delete_save(&mut self) {
        if self.storage.exists(SAVE_KEY) {
            self.storage
                .remove_raw(SAVE_KEY)
                .expect("Failed to clear save state");
        }
    }
    pub fn tick<I>(&mut self, inputs: I, period: Duration, view: &AppView) -> Option<Tick>
    where
        I: IntoIterator<Item = ProtottyInput>,
    {
        match self.app_state {
            AppState::ViewCursor => {
                // TODO centralise these dimensions
                if let Some(game_state) = self.game_state.as_ref() {
                    let to_render = game_state.game.to_render();
                    let size = Size::new(60, 40);
                    let view_cursor = if let Some(view_cursor) = self.view_cursor {
                        view_cursor
                    } else {
                        to_render.player.coord()
                    };
                    let view_cursor = if let Some(input) = inputs.into_iter().next() {
                        let delta = match input {
                            ProtottyInput::Up => Some(Coord::new(0, -1)),
                            ProtottyInput::Down => Some(Coord::new(0, 1)),
                            ProtottyInput::Left => Some(Coord::new(-1, 0)),
                            ProtottyInput::Right => Some(Coord::new(1, 0)),
                            Input::MouseMove { .. } => None,
                            prototty_inputs::ETX => return Some(Tick::Quit),
                            _ => {
                                self.app_state = AppState::Game;
                                None
                            }
                        };
                        if let Some(delta) = delta {
                            let next_cursor = view_cursor + delta;
                            if next_cursor.is_valid(size) {
                                next_cursor
                            } else {
                                view_cursor
                            }
                        } else {
                            view_cursor
                        }
                    } else {
                        view_cursor
                    };
                    self.view_cursor = Some(view_cursor);
                    if let Some(cell) = to_render.world.grid().get(view_cursor) {
                        if to_render.visible_area.is_visible(view_cursor)
                            && to_render.visible_area.light_colour(view_cursor)
                                != grey24(0)
                        {
                            use gws::*;
                            if let Some(foreground_tile) =
                                cell.foreground_tiles(to_render.world.entities()).next()
                            {
                                self.message = match foreground_tile {
                                    ForegroundTile::Bumper => Some("Bumper".to_string()),
                                    ForegroundTile::Caster => Some("Caster".to_string()),
                                    ForegroundTile::Healer => Some("Healer".to_string()),
                                    ForegroundTile::Spike => Some("Spike".to_string()),
                                    ForegroundTile::Spark => None,
                                    ForegroundTile::Blink0 => None,
                                    ForegroundTile::Blink1 => None,
                                    ForegroundTile::Player => Some("You".to_string()),
                                    ForegroundTile::Tree => Some("Tree".to_string()),
                                    ForegroundTile::Block => Some("Block".to_string()),
                                    ForegroundTile::Stairs => {
                                        Some("Stairs to the next level".to_string())
                                    }
                                    ForegroundTile::Flame => {
                                        Some("Cleansing Flame".to_string())
                                    }
                                    ForegroundTile::Altar => {
                                        Some("Cursed Altar".to_string())
                                    }
                                    ForegroundTile::Fountain => {
                                        Some("Plentiful Fountain".to_string())
                                    }
                                }
                            } else {
                                self.message = match cell.background_tile() {
                                    BackgroundTile::Floor => Some("Floor".to_string()),
                                    BackgroundTile::Ground => Some("Ground".to_string()),
                                    BackgroundTile::IceWall => {
                                        Some("Ice Wall".to_string())
                                    }
                                };
                            }
                        } else {
                            self.message = None;
                        }
                    }
                }
            }
            AppState::ListDeck
            | AppState::ListSpent
            | AppState::ListWaste
            | AppState::ListBurnt => {
                for input in inputs {
                    match input {
                        Input::MouseMove { .. } => (),
                        prototty_inputs::ETX => return Some(Tick::Quit),
                        _other => {
                            self.app_state = AppState::Game;
                        }
                    }
                }
            }
            AppState::AltarMenu => {
                match self
                    .altar_menu
                    .as_mut()
                    .unwrap()
                    .tick_with_mouse(inputs, &view.altar_view.menu_view)
                {
                    None => (),
                    Some(MenuOutput::Cancel) => {
                        self.app_state = AppState::Game;
                        self.altar_menu = None;
                    }
                    Some(MenuOutput::Quit) => return Some(Tick::Quit),
                    Some(MenuOutput::Finalise((character_upgrade, card))) => {
                        let game_state = self.game_state.as_mut().unwrap();
                        let input_start_index = game_state.all_inputs.len();
                        let interactive = self.interactive.unwrap();
                        let entity_id = interactive.entity_id;
                        game_state.all_inputs.push(gws::input::interact(
                            gws::InteractiveParam::Altar {
                                entity_id,
                                card,
                                character_upgrade,
                            },
                        ));
                        let input_end_index = game_state.all_inputs.len();
                        let _ = game_state.game.tick(
                            game_state.all_inputs[input_start_index..input_end_index]
                                .into_iter()
                                .cloned(),
                            period,
                            &mut game_state.rng_with_seed.rng,
                        );
                        self.app_state = AppState::Game;
                        self.interactive = None;
                        self.altar_menu = None;
                    }
                }
            }
            AppState::FountainMenu => {
                match self
                    .fountain_menu
                    .as_mut()
                    .unwrap()
                    .tick_with_mouse(inputs, &view.fountain_view.menu_view)
                {
                    None => (),
                    Some(MenuOutput::Cancel) => {
                        self.app_state = AppState::Game;
                        self.fountain_menu = None;
                    }
                    Some(MenuOutput::Quit) => return Some(Tick::Quit),
                    Some(MenuOutput::Finalise((card, count))) => {
                        let game_state = self.game_state.as_mut().unwrap();
                        let input_start_index = game_state.all_inputs.len();
                        let interactive = self.interactive.unwrap();
                        let entity_id = interactive.entity_id;
                        game_state.all_inputs.push(gws::input::interact(
                            gws::InteractiveParam::Fountain {
                                entity_id,
                                card,
                                count,
                            },
                        ));
                        let input_end_index = game_state.all_inputs.len();
                        let _ = game_state.game.tick(
                            game_state.all_inputs[input_start_index..input_end_index]
                                .into_iter()
                                .cloned(),
                            period,
                            &mut game_state.rng_with_seed.rng,
                        );
                        self.app_state = AppState::Game;
                        self.interactive = None;
                        self.fountain_menu = None;
                    }
                }
            }
            AppState::CardMenu => {
                match self
                    .card_menu
                    .as_mut()
                    .unwrap()
                    .tick_with_mouse(inputs, &view.flame_view.menu_view)
                {
                    None => (),
                    Some(MenuOutput::Cancel) => {
                        self.app_state = AppState::Game;
                        self.card_menu = None;
                    }
                    Some(MenuOutput::Quit) => return Some(Tick::Quit),
                    Some(MenuOutput::Finalise(card)) => {
                        let game_state = self.game_state.as_mut().unwrap();
                        let input_start_index = game_state.all_inputs.len();
                        let interactive = self.interactive.unwrap();
                        let entity_id = interactive.entity_id;
                        game_state.all_inputs.push(gws::input::interact(
                            gws::InteractiveParam::Flame { entity_id, card },
                        ));
                        let input_end_index = game_state.all_inputs.len();
                        let _ = game_state.game.tick(
                            game_state.all_inputs[input_start_index..input_end_index]
                                .into_iter()
                                .cloned(),
                            period,
                            &mut game_state.rng_with_seed.rng,
                        );
                        self.app_state = AppState::Game;
                        self.interactive = None;
                        self.card_menu = None;
                    }
                }
            }
            AppState::Death => {
                for input in inputs {
                    match input {
                        Input::MouseMove { .. } => (),
                        prototty_inputs::ETX => return Some(Tick::Quit),
                        _other => {
                            self.app_state = AppState::Menu;
                            self.game_state = None;
                            self.delete_save();
                        }
                    }
                }
            }
            AppState::Menu => {
                if self.game_state.is_some() {
                    match self
                        .pause_menu
                        .tick_with_mouse(inputs, &view.menu_and_title_view.menu_view)
                    {
                        None => (),
                        Some(MenuOutput::Cancel) => {
                            self.app_state = AppState::Game;
                        }
                        Some(MenuOutput::Quit) => return Some(Tick::Quit),
                        Some(MenuOutput::Finalise(selection)) => match selection {
                            pause_menu::Choice::Resume => {
                                self.app_state = AppState::Game;
                            }
                            pause_menu::Choice::SaveAndQuit => {
                                self.save();
                                return Some(Tick::Quit);
                            }
                            pause_menu::Choice::NewGame => {
                                self.app_state = AppState::BetweenLevels(None);
                            }
                            pause_menu::Choice::Help => {
                                self.app_state = AppState::Help {
                                    opened_from_game: false,
                                }
                            }
                            pause_menu::Choice::Map => {
                                self.app_state = AppState::Map {
                                    opened_from_game: false,
                                }
                            }
                        },
                    }
                } else {
                    match self
                        .menu
                        .tick_with_mouse(inputs, &view.menu_and_title_view.menu_view)
                    {
                        None | Some(MenuOutput::Cancel) => (),
                        Some(MenuOutput::Quit) => return Some(Tick::Quit),
                        Some(MenuOutput::Finalise(selection)) => match selection {
                            menu::Choice::Quit => return Some(Tick::Quit),
                            menu::Choice::NewGame => {
                                self.app_state = AppState::BetweenLevels(None);
                            }
                            menu::Choice::Help => {
                                self.app_state = AppState::Help {
                                    opened_from_game: false,
                                }
                            }
                        },
                    }
                }
            }
            AppState::Game => {
                if let Some(game_state) = self.game_state.as_mut() {
                    let input_start_index = game_state.all_inputs.len();
                    if let Some(CardInSlot {
                        slot,
                        ref mut choice,
                    }) = self.card_selection.as_mut()
                    {
                        if let Some(input) = inputs.into_iter().next() {
                            let slot = *slot; // TODO why is this necessary
                            let game_input = match input {
                                Input::MouseMove { .. } => None,
                                prototty_inputs::ESCAPE => {
                                    self.message = None;
                                    self.card_selection = None;
                                    None
                                }
                                prototty_inputs::ETX => return Some(Tick::Quit),
                                ProtottyInput::Char(card_num @ '1'...'8') => {
                                    let (message, card_selection) =
                                        Self::select_card(game_state, card_num);
                                    self.message = message;
                                    self.card_selection = card_selection;
                                    None
                                }
                                _ => match *choice {
                                    CardParamChoice::Confirm => match input {
                                        prototty_inputs::RETURN => {
                                            Some(gws::input::play_card(
                                                slot,
                                                gws::CardParam::Confirm,
                                            ))
                                        }
                                        _ => {
                                            self.message = None;
                                            self.card_selection = None;
                                            None
                                        }
                                    },
                                    CardParamChoice::Coord(coord) => match input {
                                        ProtottyInput::Up => {
                                            *choice = CardParamChoice::Coord(
                                                coord + Coord::new(0, -1),
                                            );
                                            None
                                        }
                                        ProtottyInput::Down => {
                                            *choice = CardParamChoice::Coord(
                                                coord + Coord::new(0, 1),
                                            );
                                            None
                                        }
                                        ProtottyInput::Left => {
                                            *choice = CardParamChoice::Coord(
                                                coord + Coord::new(-1, 0),
                                            );
                                            None
                                        }
                                        ProtottyInput::Right => {
                                            *choice = CardParamChoice::Coord(
                                                coord + Coord::new(1, 0),
                                            );
                                            None
                                        }
                                        prototty_inputs::RETURN => {
                                            Some(gws::input::play_card(
                                                slot,
                                                gws::CardParam::Coord(coord),
                                            ))
                                        }
                                        _ => {
                                            self.message = None;
                                            self.card_selection = None;
                                            None
                                        }
                                    },
                                    CardParamChoice::Direction => match input {
                                        ProtottyInput::Up => Some(gws::input::play_card(
                                            slot,
                                            gws::CardParam::CardinalDirection(
                                                CardinalDirection::North,
                                            ),
                                        )),
                                        ProtottyInput::Down => {
                                            Some(gws::input::play_card(
                                                slot,
                                                gws::CardParam::CardinalDirection(
                                                    CardinalDirection::South,
                                                ),
                                            ))
                                        }
                                        ProtottyInput::Left => {
                                            Some(gws::input::play_card(
                                                slot,
                                                gws::CardParam::CardinalDirection(
                                                    CardinalDirection::West,
                                                ),
                                            ))
                                        }
                                        ProtottyInput::Right => {
                                            Some(gws::input::play_card(
                                                slot,
                                                gws::CardParam::CardinalDirection(
                                                    CardinalDirection::East,
                                                ),
                                            ))
                                        }
                                        _ => {
                                            self.message = None;
                                            self.card_selection = None;
                                            None
                                        }
                                    },
                                },
                            };
                            if let Some(game_input) = game_input {
                                game_state.all_inputs.push(game_input);
                            }
                        }
                    } else {
                        for input in inputs {
                            match input {
                                ProtottyInput::Up => {
                                    game_state.all_inputs.push(gws::input::UP)
                                }
                                ProtottyInput::Down => {
                                    game_state.all_inputs.push(gws::input::DOWN)
                                }
                                ProtottyInput::Left => {
                                    game_state.all_inputs.push(gws::input::LEFT)
                                }
                                ProtottyInput::Right => {
                                    game_state.all_inputs.push(gws::input::RIGHT)
                                }
                                ProtottyInput::Char(' ') => {
                                    game_state.all_inputs.push(gws::input::WAIT)
                                }
                                MAP_INPUT0 | MAP_INPUT1 => {
                                    self.app_state = AppState::Map {
                                        opened_from_game: true,
                                    }
                                }
                                HELP_INPUT0 | HELP_INPUT1 => {
                                    self.app_state = AppState::Help {
                                        opened_from_game: true,
                                    }
                                }
                                ProtottyInput::Char('d') => {
                                    self.app_state = AppState::ListDeck;
                                }
                                ProtottyInput::Char('s') => {
                                    self.app_state = AppState::ListSpent;
                                }
                                ProtottyInput::Char('w') => {
                                    self.app_state = AppState::ListWaste;
                                }
                                ProtottyInput::Char('b') => {
                                    self.app_state = AppState::ListBurnt;
                                }
                                ProtottyInput::Char('v') => {
                                    self.view_cursor =
                                        Some(game_state.game.to_render().player.coord());
                                    self.app_state = AppState::ViewCursor;
                                }
                                ProtottyInput::Char(card_num @ '1'...'8') => {
                                    let (message, card_selection) =
                                        Self::select_card(game_state, card_num);
                                    self.message = message;
                                    self.card_selection = card_selection;
                                }
                                prototty_inputs::ESCAPE => {
                                    self.app_state = AppState::Menu
                                }
                                prototty_inputs::ETX => return Some(Tick::Quit),
                                _ => (),
                            }
                        }
                    }
                    let input_end_index = game_state.all_inputs.len();
                    if input_end_index != input_start_index {
                        self.message = None;
                        self.card_selection = None;
                    }
                    let tick = game_state.game.tick(
                        game_state.all_inputs[input_start_index..input_end_index]
                            .into_iter()
                            .cloned(),
                        period,
                        &mut game_state.rng_with_seed.rng,
                    );
                    if let Some(tick) = tick {
                        match tick {
                            gws::Tick::Interact(interactive) => {
                                self.interactive = Some(interactive);
                                match interactive.typ {
                                    gws::InteractiveType::Flame => {
                                        let spent = game_state.game.spent();
                                        if spent.is_empty() {
                                            self.message = Some("You must have spent cards to interact with the Cleansing Flame.".to_string());
                                        } else {
                                            self.card_menu_title =
                                                "Cleansing Flame: Burn a spent card. Lose 1 life.".to_string();
                                            self.card_menu = menus::card_menu::create(
                                                spent,
                                                &self.card_table,
                                            );
                                            self.app_state = AppState::CardMenu;
                                        }
                                    }
                                    gws::InteractiveType::Altar => {
                                        self.altar_menu =
                                            Some(menus::altar_menu::create(
                                                &game_state.game,
                                                &self.card_table,
                                                &mut game_state.rng_with_seed.rng,
                                            ));
                                        self.app_state = AppState::AltarMenu;
                                    }
                                    gws::InteractiveType::Fountain => {
                                        self.fountain_menu =
                                            Some(menus::fountain_menu::create(
                                                &game_state.game,
                                                &self.card_table,
                                                &mut game_state.rng_with_seed.rng,
                                            ));
                                        self.app_state = AppState::FountainMenu;
                                    }
                                }
                            }
                            gws::Tick::End(end) => match end {
                                gws::End::ExitLevel(between_levels) => {
                                    self.app_state =
                                        AppState::BetweenLevels(Some(between_levels));
                                    self.card_selection = None;
                                    self.message = None;
                                }
                                gws::End::PlayerDied => {
                                    self.save();
                                    self.app_state = AppState::Death;
                                }
                            },
                            gws::Tick::CancelAction(cancel_action) => {
                                use gws::CancelAction::*;
                                match cancel_action {
                                    MoveIntoSolidCell | OutOfBounds => {
                                        self.message =
                                            Some("Can't move there!".to_string())
                                    }
                                    LocationBlocked => {
                                        self.message =
                                            Some("Location is blocked".to_string())
                                    }
                                    OutOfRange => {
                                        self.message = Some("Out of range!".to_string())
                                    }
                                    NothingToAttack => {
                                        self.message = Some("Nothing there!".to_string())
                                    }
                                    AlreadyFullHitPoints => {
                                        self.message =
                                            Some("Health is already full!".to_string())
                                    }
                                    DestinationNotVisible => {
                                        self.message =
                                            Some("Can't see there!".to_string())
                                    }
                                    NotEnoughEnergy => {
                                        self.message =
                                            Some("Not enough power!".to_string())
                                    }
                                    _ => (),
                                }
                            }
                        }
                    }
                } else {
                    self.app_state = AppState::Menu;
                }
            }
            AppState::Map { opened_from_game } => {
                for input in inputs {
                    match input {
                        prototty_inputs::ESCAPE | MAP_INPUT0 | MAP_INPUT1 => {
                            if opened_from_game {
                                self.app_state = AppState::Game
                            } else {
                                self.app_state = AppState::Menu
                            }
                        }
                        HELP_INPUT0 | HELP_INPUT1 => {
                            self.app_state = AppState::Help { opened_from_game }
                        }
                        prototty_inputs::ETX => return Some(Tick::Quit),
                        _ => (),
                    }
                }
            }
            AppState::Help { opened_from_game } => {
                for input in inputs {
                    match input {
                        prototty_inputs::ESCAPE | HELP_INPUT0 | HELP_INPUT1 => {
                            if opened_from_game {
                                self.app_state = AppState::Game
                            } else {
                                self.app_state = AppState::Menu
                            }
                        }
                        MAP_INPUT0 | MAP_INPUT1 => {
                            self.app_state = AppState::Map { opened_from_game }
                        }
                        prototty_inputs::ETX => return Some(Tick::Quit),
                        _ => (),
                    }
                }
            }
            AppState::BetweenLevels(ref between_levels) => {
                let rng_with_seed = self.rng_source.next();
                let seed = rng_with_seed.seed;
                let first_level = between_levels.is_none();
                self.game_state = Some(GameState::new(
                    between_levels.clone(),
                    rng_with_seed,
                    self.debug_terrain_string.as_ref().map(String::as_str),
                ));
                self.app_state = AppState::Game;
                if first_level {
                    return Some(Tick::GameInitialisedWithSeed(seed));
                }
            }
        }
        if let Some(time_until_next_auto_save) =
            self.time_until_next_auto_save.checked_sub(period)
        {
            self.time_until_next_auto_save = time_until_next_auto_save;
            None
        } else {
            self.time_until_next_auto_save = AUTO_SAVE_PERIOD;
            self.save();
            Some(Tick::AutoSave)
        }
    }
    fn select_card(
        game_state: &GameState,
        card_num: char,
    ) -> (Option<String>, Option<CardInSlot>) {
        let card_index = card_num.to_digit(10).unwrap() as usize - 1;
        let hand = game_state.game.hand();
        let message;
        let card_selection;
        if let Some(&maybe_card) = hand.get(card_index) {
            if let Some(card) = maybe_card {
                if game_state.game.draw_countdown().current < card.cost() {
                    message = Some("Not enough power!".to_string());
                    card_selection = None;
                } else {
                    let choice = match card {
                        gws::Card::Bump | gws::Card::Spark => {
                            message = Some("Choose a direction.".to_string());
                            CardParamChoice::Direction
                        }
                        gws::Card::Blink
                        | gws::Card::Block
                        | gws::Card::Freeze
                        | gws::Card::Spike => {
                            message = Some("Choose a location.".to_string());
                            CardParamChoice::Coord(
                                game_state.game.to_render().player.coord(),
                            )
                        }
                        gws::Card::Heal
                        | gws::Card::Clog
                        | gws::Card::Parasite
                        | gws::Card::Drain => {
                            message = Some("Confirm selection.".to_string());
                            CardParamChoice::Confirm
                        }
                    };
                    card_selection = Some(CardInSlot {
                        slot: card_index,
                        choice,
                    });
                }
            } else {
                message = Some(format!("No card in slot {}.", card_num));
                card_selection = None;
            }
        } else {
            message = Some(format!("Card slot {} is locked.", card_num));
            card_selection = None;
        }
        (message, card_selection)
    }
}

impl AppView {
    pub fn new() -> Self {
        Self {
            menu_and_title_view: MenuAndTitleView::new(TITLE_COLOUR),
            flame_view: MenuAndTitleView::new(rgb24(255, 120, 0)),
            altar_view: MenuAndTitleView::new(rgb24(0, 200, 50)),
            fountain_view: MenuAndTitleView::new(rgb24(50, 100, 200)),
        }
    }
    pub fn set_size(&mut self, _size: Size) {}
}
