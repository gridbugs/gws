#[macro_use]
extern crate simon;
extern crate cherenkov_prototty;
extern crate whoami;

use cherenkov_prototty::FirstRngSeed;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

const SAVE_BASE: &'static str = "user";

pub struct CommonArgs {
    rng_seed: FirstRngSeed,
    name: String,
    debug_terrain_file: Option<String>,
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
                debug_terrain_file = simon::opt("t", "debug-terain-file",
                                                "text file to influence terrain generation",
                                                "FILE");

            } in {
                Self { rng_seed, name, debug_terrain_file }
            }
        }
    }
    pub fn save_dir(&self) -> PathBuf {
        Path::new(SAVE_BASE).join(&self.name)
    }
    pub fn first_rng_seed(&self) -> FirstRngSeed {
        self.rng_seed
    }
    pub fn debug_terrain_string(&self) -> Option<String> {
        self.debug_terrain_file.as_ref().map(|filename| {
            let mut f = File::open(filename).unwrap();
            let mut buffer = String::new();
            f.read_to_string(&mut buffer).unwrap();
            buffer
        })
    }
}
