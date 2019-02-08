extern crate cherenkov;
extern crate prototty;
extern crate rand;
extern crate rand_isaac;
#[macro_use]
extern crate serde;

pub mod frontend;
pub mod menus;

use menus::*;
use prototty::*;
use rand::{FromEntropy, Rng, SeedableRng};
use rand_isaac::IsaacRng;
use std::time::Duration;

const TITLE: &'static str = "CHERENKOV";

pub struct AppView {
    menu_and_title_view: MenuAndTitleView,
}

pub enum Tick {
    Quit,
    GameInitialisedWithSeed(u64),
    GameSaved,
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
}

impl GameState {
    fn new(rng_with_seed: RngWithSeed) -> Self {
        Self { rng_with_seed }
    }
}

enum AppState {
    GameInProgress(GameState),
    PauseMenu(GameState),
    Menu,
}

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
    frontend: F,
    storage: S,
    app_state: AppState,
    rng_source: RngSource,
    menu: MenuInstance<menu::Choice>,
    pause_menu: MenuInstance<pause_menu::Choice>,
}

impl<F: Frontend, S: Storage> View<App<F, S>> for AppView {
    fn view<G>(&mut self, app: &App<F, S>, offset: Coord, depth: i32, grid: &mut G)
    where
        G: ViewGrid,
    {
        match app.app_state {
            AppState::Menu => self.menu_and_title_view.view(
                &MenuAndTitle::new(&app.menu, TITLE),
                offset + Coord::new(1, 1),
                depth,
                grid,
            ),
            AppState::PauseMenu(_) => (),
            AppState::GameInProgress(_) => (),
        }
    }
}

impl<F: Frontend, S: Storage> App<F, S> {
    pub fn new(frontend: F, storage: S, first_rng_seed: FirstRngSeed) -> (Self, InitStatus) {
        let (init_status, app_state) = match storage.load::<_, GameState>(SAVE_KEY) {
            Ok(game_state) => (
                InitStatus::LoadedSaveWithSeed(game_state.rng_with_seed.seed),
                AppState::PauseMenu(game_state),
            ),
            Err(_) => (InitStatus::NoSaveFound, AppState::Menu),
        };
        let rng_source = RngSource::new(first_rng_seed);
        let menu = menu::create();
        let pause_menu = pause_menu::create();
        let app = Self {
            frontend,
            storage,
            app_state,
            rng_source,
            menu,
            pause_menu,
        };
        (app, init_status)
    }
    pub fn tick<I>(&mut self, inputs: I, _period: Duration, view: &AppView) -> Option<Tick>
    where
        I: IntoIterator<Item = ProtottyInput>,
    {
        match self.app_state {
            AppState::Menu => {
                match self
                    .menu
                    .tick_with_mouse(inputs, &view.menu_and_title_view.menu_view)
                {
                    None | Some(MenuOutput::Cancel) => (),
                    Some(MenuOutput::Quit) => return Some(Tick::Quit),
                    Some(MenuOutput::Finalise(selection)) => match selection {
                        menu::Choice::Quit => return Some(Tick::Quit),
                        menu::Choice::NewGame => (),
                    },
                }
            }
            AppState::PauseMenu(_) => (),
            AppState::GameInProgress(_) => (),
        }
        None
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
