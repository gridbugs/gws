extern crate gws_native;
extern crate gws_prototty;
extern crate prototty_glutin;
extern crate simon;

use gws_native::*;
use gws_prototty::*;
use prototty_glutin::*;
use simon::*;
use std::time::Instant;

struct Args {
    common: CommonArgs,
    font_size: u32,
}

const DEFAULT_FONT_SIZE: u32 = 10;

impl Args {
    fn arg() -> ArgExt<impl Arg<Item = Self>> {
        args_map! {
            let {
                common = CommonArgs::arg();
                font_size = opt("s", "font-size", "font size in pixels", "INT")
                    .with_default(DEFAULT_FONT_SIZE);
            } in {
                Self { common, font_size }
            }
        }
    }
}

fn main() {
    let args = Args::arg().with_help_default().parse_env_default_or_exit();
    let grid_size = gws_prototty::APP_SIZE;
    let font_size = args.font_size;
    let size = grid_size * font_size;
    let mut context =
        ContextBuilder::new_with_font(include_bytes!("fonts/PxPlus_IBM_CGAthin.ttf"))
            .with_bold_font(include_bytes!("fonts/PxPlus_IBM_CGA.ttf"))
            .with_window_dimensions(size)
            .with_min_window_dimensions(size)
            .with_max_window_dimensions(size)
            .with_font_scale(font_size as f32, font_size as f32)
            .with_cell_dimensions(Size::new(font_size, font_size))
            .with_max_grid_size(grid_size)
            .with_underline_width(2)
            .with_underline_position(font_size - 2)
            .with_title("Get Well Soon")
            .build()
            .unwrap();
    let storage = FileStorage::next_to_exe(args.common.save_dir(), true)
        .expect("Failed to find user dir");
    let (mut app, init_status) = App::new(
        frontend::Glutin,
        storage,
        args.common.first_rng_seed(),
        args.common.debug_terrain_string(),
    );
    let mut input_buffer = Vec::with_capacity(64);
    let mut app_view = AppView::new();
    let mut frame_instant = Instant::now();
    match init_status {
        InitStatus::NoSaveFound => eprintln!("No save game found"),
        InitStatus::LoadedSaveWithSeed(seed) => {
            eprintln!("Loaded game with seed: {}", seed)
        }
    }
    loop {
        let period = frame_instant.elapsed();
        frame_instant = Instant::now();
        context.buffer_input(&mut input_buffer);
        if let Some(tick) = app.tick(input_buffer.drain(..), period, &app_view) {
            match tick {
                Tick::Quit => break,
                Tick::GameInitialisedWithSeed(seed) => {
                    eprintln!("Initialised game with seed: {}", seed)
                }
                Tick::AutoSave => (),
            }
        }
        app_view.set_size(context.size());
        context.render(&mut app_view, &app).unwrap();
    }
}
