extern crate cherenkov_prototty;
extern crate prototty_glutin;

use cherenkov_prototty::{App, AppView, Quit};
use prototty_glutin::*;
use std::time::Instant;

fn main() {
    let size = Size::new(960, 720);
    let mut context = ContextBuilder::new_with_font(include_bytes!("fonts/PxPlus_IBM_CGAthin.ttf"))
        .with_bold_font(include_bytes!("fonts/PxPlus_IBM_CGA.ttf"))
        .with_window_dimensions(size)
        .with_min_window_dimensions(size)
        .with_max_window_dimensions(size)
        .with_window_dimensions(Size::new(640, 480))
        .with_font_scale(16.0, 16.0)
        .with_cell_dimensions(Size::new(16, 16))
        .with_max_grid_size(Size::new(80, 60))
        .build()
        .unwrap();
    let mut app = App::new();
    let mut input_buffer = Vec::with_capacity(64);
    let mut app_view = AppView::new();
    let mut last_frame = Instant::now();
    loop {
        let frame_start = Instant::now();
        let period = frame_start - last_frame;
        last_frame = frame_start;
        context.buffer_input(&mut input_buffer);
        if let Some(Quit) = app.tick(input_buffer.drain(..), period) {
            break;
        }
        app_view.set_size(context.size());
        context.render(&mut app_view, &app).unwrap();
    }
}
