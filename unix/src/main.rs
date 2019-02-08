extern crate cherenkov_prototty;
extern crate prototty_unix;

use cherenkov_prototty::*;
use prototty_unix::*;
use std::thread;
use std::time::{Duration, Instant};

const USER_DIR: &'static str = "user";
const TARGET_FPS: f64 = 60.;
const TICK_PERIOD: Duration = Duration::from_micros((1_000_000. / TARGET_FPS) as u64);

fn main() {
    let mut context = Context::new().unwrap();
    let storage = FileStorage::next_to_exe(USER_DIR, true).expect("Failed to find user dir");
    let (mut app, _init_status) = App::new(frontend::Unix, storage, FirstRngSeed::Random);
    let mut app_view = AppView::new();
    let mut last_frame = Instant::now();
    loop {
        let frame_start = Instant::now();
        let period = frame_start - last_frame;
        last_frame = frame_start;
        if let Some(tick) = app.tick(context.drain_input().unwrap(), period, &app_view) {
            match tick {
                Tick::Quit => break,
                Tick::GameInitialisedWithSeed(_) | Tick::GameSaved => (),
            }
        }
        app_view.set_size(context.size().unwrap());
        context.render(&mut app_view, &app).unwrap();
        let time_until_next_frame = TICK_PERIOD - (Instant::now() - frame_start);
        thread::sleep(time_until_next_frame);
    }
}
