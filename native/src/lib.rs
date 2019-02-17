#[macro_use]
extern crate simon;
extern crate cherenkov_prototty;
extern crate whoami;

use cherenkov_prototty::FirstRngSeed;
use std::path::{Path, PathBuf};

const SAVE_BASE: &'static str = "user";

pub struct CommonArgs {
    rng_seed: FirstRngSeed,
    name: String,
}

impl CommonArgs {
    pub fn arg() -> simon::ArgExt<impl simon::Arg<Item = Self>> {
        args_map! {
            let {
                rng_seed = simon::opt(
                    "r",
                    "rng-seed",
                    "seed to use to generate first new game",
                    "INT",
                )
                .option_map(FirstRngSeed::Seed)
                .with_default(FirstRngSeed::Random);
                name = simon::opt("n", "name", "name to use for save game", "NAME")
                    .map(|n| n.unwrap_or_else(|| whoami::username()));
            } in {
                Self { rng_seed, name }
            }
        }
    }
    pub fn save_dir(&self) -> PathBuf {
        Path::new(SAVE_BASE).join(&self.name)
    }
    pub fn first_rng_seed(&self) -> FirstRngSeed {
        self.rng_seed
    }
}
