extern crate cherenkov;
extern crate prototty;
use prototty::*;
use std::time::Duration;

pub struct App;
pub struct AppView;
pub struct Quit;

impl View<App> for AppView {
    fn view<G>(&mut self, _data: &App, offset: Coord, depth: i32, grid: &mut G)
    where
        G: ViewGrid,
    {
        StringView.view("It works!", offset, depth, grid);
    }
}

impl App {
    pub fn new() -> Self {
        App
    }
    pub fn tick<I>(&mut self, i: I, _period: Duration) -> Option<Quit>
    where
        I: IntoIterator<Item = ProtottyInput>,
    {
        for input in i.into_iter() {
            match input {
                prototty_inputs::ETX => return Some(Quit),
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
