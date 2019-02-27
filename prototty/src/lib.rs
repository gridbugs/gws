extern crate cherenkov;
extern crate prototty;
extern crate rand;
extern crate rand_isaac;
#[macro_use]
extern crate serde;

pub mod frontend;
mod game_view;
mod map_view;
mod menus;

use game_view::GameView;
use map_view::MapView;
use menus::*;
use prototty::*;
use rand::{FromEntropy, Rng, SeedableRng};
use rand_isaac::IsaacRng;
use std::marker::PhantomData;
use std::time::Duration;

const TITLE: &'static str = "CHERENKOV";
const AUTO_SAVE_PERIOD: Duration = Duration::from_millis(5000);

const APP_SIZE: Size = Size::new_u16(64, 48);

pub struct AppView {
    menu_and_title_view: MenuAndTitleView,
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
    all_inputs: Vec<cherenkov::Input>,
    game: cherenkov::Cherenkov,
}

impl GameState {
    fn new(mut rng_with_seed: RngWithSeed) -> Self {
        let game = cherenkov::Cherenkov::new(&mut rng_with_seed.rng);
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
    Map,
    Help { opened_from_game: bool },
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
}

impl<F: Frontend, S: Storage> View<App<F, S>> for AppView {
    fn view<G>(&mut self, app: &App<F, S>, offset: Coord, depth: i32, grid: &mut G)
    where
        G: ViewGrid,
    {
        match app.app_state {
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
                    GameView.view(&game_state.game, offset, depth, grid);
                }
            }
            AppState::Map => {
                if let Some(game_state) = app.game_state.as_ref() {
                    MapView.view(&game_state.game, offset, depth, grid);
                }
            }
            AppState::Help { .. } => {
                PagerView.view(&app.help_pager, offset, depth, grid);
            }
        }
    }
}

const HELP_INPUT0: ProtottyInput = ProtottyInput::Char('h');
const HELP_INPUT1: ProtottyInput = ProtottyInput::Function(1);
const MAP_INPUT0: ProtottyInput = ProtottyInput::Char('m');
const MAP_INPUT1: ProtottyInput = ProtottyInput::Function(2);

impl<F: Frontend, S: Storage> App<F, S> {
    pub fn new(frontend: F, storage: S, first_rng_seed: FirstRngSeed) -> (Self, InitStatus) {
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
            help_pager: Pager::new(include_str!("help.txt"), APP_SIZE, Default::default()),
        };
        (app, init_status)
    }
    pub fn save(&mut self) {
        if let Some(game_state) = self.game_state.as_ref() {
            self.storage
                .store(SAVE_KEY, &game_state)
                .expect("Failed to save game");
        }
    }
    pub fn tick<I>(&mut self, inputs: I, period: Duration, view: &AppView) -> Option<Tick>
    where
        I: IntoIterator<Item = ProtottyInput>,
    {
        match self.app_state {
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
                                self.game_state = Some(GameState::new(self.rng_source.next()));
                                self.app_state = AppState::Game;
                            }
                            pause_menu::Choice::Help => {
                                self.app_state = AppState::Help {
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
                                self.game_state = Some(GameState::new(self.rng_source.next()));
                                self.app_state = AppState::Game;
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
                    for input in inputs {
                        match input {
                            ProtottyInput::Up => game_state.all_inputs.push(cherenkov::input::UP),
                            ProtottyInput::Down => {
                                game_state.all_inputs.push(cherenkov::input::DOWN)
                            }
                            ProtottyInput::Left => {
                                game_state.all_inputs.push(cherenkov::input::LEFT)
                            }
                            ProtottyInput::Right => {
                                game_state.all_inputs.push(cherenkov::input::RIGHT)
                            }
                            MAP_INPUT0 | MAP_INPUT1 => self.app_state = AppState::Map,
                            HELP_INPUT0 | HELP_INPUT1 => {
                                self.app_state = AppState::Help {
                                    opened_from_game: true,
                                }
                            }
                            prototty_inputs::ESCAPE => self.app_state = AppState::Menu,
                            prototty_inputs::ETX => return Some(Tick::Quit),
                            _ => (),
                        }
                    }
                    let input_end_index = game_state.all_inputs.len();
                    game_state.game.tick(
                        game_state.all_inputs[input_start_index..input_end_index]
                            .into_iter()
                            .cloned(),
                        &mut game_state.rng_with_seed.rng,
                    );
                } else {
                    self.app_state = AppState::Menu;
                }
            }
            AppState::Map => {
                for input in inputs {
                    match input {
                        prototty_inputs::ESCAPE | MAP_INPUT0 | MAP_INPUT1 => {
                            self.app_state = AppState::Game
                        }
                        HELP_INPUT0 | HELP_INPUT1 => {
                            self.app_state = AppState::Help {
                                opened_from_game: true,
                            }
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
                        MAP_INPUT0 | MAP_INPUT1 => self.app_state = AppState::Map,
                        prototty_inputs::ETX => return Some(Tick::Quit),
                        _ => (),
                    }
                }
            }
        }
        if let Some(time_until_next_auto_save) = self.time_until_next_auto_save.checked_sub(period)
        {
            self.time_until_next_auto_save = time_until_next_auto_save;
            None
        } else {
            self.time_until_next_auto_save = AUTO_SAVE_PERIOD;
            self.save();
            Some(Tick::AutoSave)
        }
    }
}

impl AppView {
    pub fn new() -> Self {
        Self {
            menu_and_title_view: MenuAndTitleView::new(),
        }
    }
    pub fn set_size(&mut self, _size: Size) {}
}
