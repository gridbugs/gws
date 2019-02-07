extern crate cherenkov;
extern crate prototty;
extern crate rand;
extern crate rand_isaac;
#[macro_use]
extern crate serde;

use prototty::*;
use rand::{FromEntropy, Rng, SeedableRng};
use rand_isaac::IsaacRng;
use std::time::Duration;

pub mod frontend;

pub struct AppView;

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
}

impl<F: Frontend, S: Storage> View<App<F, S>> for AppView {
    fn view<G>(&mut self, _data: &App<F, S>, offset: Coord, depth: i32, grid: &mut G)
    where
        G: ViewGrid,
    {
        StringView.view("It works!", offset, depth, grid);
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
        let app = Self {
            frontend,
            storage,
            app_state,
            rng_source,
        };
        (app, init_status)
    }
    pub fn tick<I>(&mut self, i: I, _period: Duration) -> Option<Tick>
    where
        I: IntoIterator<Item = ProtottyInput>,
    {
        for input in i.into_iter() {
            match input {
                prototty_inputs::ETX => return Some(Tick::Quit),
                _ => (),
            }
        }
        None
    }
}

impl AppView {
    pub fn new() -> Self {
        AppView
    }
    pub fn set_size(&mut self, _size: Size) {}
}
