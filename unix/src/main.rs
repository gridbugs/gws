extern crate cherenkov_native;
extern crate cherenkov_prototty;
extern crate prototty_unix;

use cherenkov_native::*;
use cherenkov_prototty::*;
use prototty_unix::*;
use std::thread;
use std::time::{Duration, Instant};

const TARGET_FPS: f64 = 60.;
const TICK_PERIOD: Duration = Duration::from_micros((1_000_000. / TARGET_FPS) as u64);

struct CherenkovColourConfig;
impl ColourConfig for CherenkovColourConfig {
    fn convert_foreground_rgb24(&mut self, rgb24: Rgb24) -> AnsiColour {
        AnsiColour::from_rgb24(rgb24.saturating_scalar_mul_div(5, 3))
    }
    fn convert_background_rgb24(&mut self, rgb24: Rgb24) -> AnsiColour {
        AnsiColour::from_rgb24(rgb24.saturating_scalar_mul_div(5, 2))
    }
    fn default_foreground(&mut self) -> AnsiColour {
        AnsiColour::from_rgb24(grey24(255))
    }
    fn default_background(&mut self) -> AnsiColour {
        AnsiColour::from_rgb24(grey24(0))
    }
}

fn main() {
    let args = CommonArgs::arg()
        .with_help_default()
        .parse_env_default_or_exit();
    let mut context = Context::with_colour_config(CherenkovColourConfig).unwrap();
    let storage =
        FileStorage::next_to_exe(args.save_dir(), true).expect("Failed to find user dir");
    let (mut app, _init_status) = App::new(
        frontend::Unix,
        storage,
        args.first_rng_seed(),
        args.debug_terrain_string(),
    );
    let mut app_view = AppView::new();
    let mut frame_instant = Instant::now();
    loop {
        let period = frame_instant.elapsed();
        frame_instant = Instant::now();
        if let Some(tick) = app.tick(context.drain_input().unwrap(), period, &app_view) {
            match tick {
                Tick::Quit => break,
                Tick::GameInitialisedWithSeed(_) | Tick::AutoSave => (),
            }
        }
        app_view.set_size(context.size().unwrap());
        context.render(&mut app_view, &app).unwrap();
        if let Some(time_until_next_frame) =
            TICK_PERIOD.checked_sub(frame_instant.elapsed())
        {
            thread::sleep(time_until_next_frame);
        }
    }
}
