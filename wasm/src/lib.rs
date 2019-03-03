extern crate gws_prototty;
extern crate prototty_wasm;
extern crate wasm_bindgen;

pub use prototty_wasm::InputBuffer;

use gws_prototty::*;
use prototty_wasm::*;
use std::time::Duration;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WebApp {
    app_view: AppView,
    app: App<frontend::Wasm, WasmStorage>,
    js_grid: JsGrid,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
impl WebApp {
    #[wasm_bindgen(constructor)]
    pub fn new(js_grid: JsGrid, js_byte_storage: JsByteStorage) -> Self {
        let storage = WasmStorage::new(js_byte_storage);
        let (app, init_status) =
            App::new(frontend::Wasm, storage, FirstRngSeed::Random, None);
        let app_view = AppView::new();
        match init_status {
            InitStatus::NoSaveFound => console_log!("No save game found"),
            InitStatus::LoadedSaveWithSeed(seed) => {
                console_log!("Loaded game with seed: {}", seed)
            }
        }
        Self {
            app_view,
            app,
            js_grid,
        }
    }

    pub fn tick(&mut self, input_buffer: &InputBuffer, period_ms: f64) {
        if let Some(tick) = self.app.tick(
            input_buffer.iter(),
            Duration::from_millis(period_ms as u64),
            &self.app_view,
        ) {
            match tick {
                Tick::Quit => console_log!("Not supported"),
                Tick::GameInitialisedWithSeed(seed) => {
                    console_log!("Initialised game with seed: {}", seed)
                }
                Tick::AutoSave => (),
            }
        }
        self.js_grid.render(&mut self.app_view, &self.app);
    }
}
