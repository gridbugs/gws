extern crate cherenkov_native;
extern crate cherenkov_prototty;
extern crate prototty_glutin;

use cherenkov_native::*;
use cherenkov_prototty::*;
use prototty_glutin::*;
use std::time::Instant;

fn main() {
    let args = CommonArgs::arg()
        .with_help_default()
        .parse_env_default_or_exit();
    let size = Size::new(960, 720);
    let mut context = ContextBuilder::new_with_font(include_bytes!("fonts/PxPlus_IBM_CGAthin.ttf"))
        .with_bold_font(include_bytes!("fonts/PxPlus_IBM_CGA.ttf"))
        .with_window_dimensions(size)
        .with_min_window_dimensions(size)
        .with_max_window_dimensions(size)
        .with_window_dimensions(Size::new(640, 480))
        .with_font_scale(16.0, 16.0)
        .with_cell_dimensions(Size::new(16, 16))
        .with_max_grid_size(Size::new(80, 40))
        .build()
        .unwrap();
    let storage = FileStorage::next_to_exe(args.save_dir(), true).expect("Failed to find user dir");
    let (mut app, init_status) = App::new(frontend::Glutin, storage, args.first_rng_seed());
    let mut input_buffer = Vec::with_capacity(64);
    let mut app_view = AppView::new();
    let mut frame_instant = Instant::now();
    match init_status {
        InitStatus::NoSaveFound => eprintln!("No save game found"),
        InitStatus::LoadedSaveWithSeed(seed) => eprintln!("Loaded game with seed: {}", seed),
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
