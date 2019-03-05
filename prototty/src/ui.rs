use crate::game_view::*;
use gws::*;
use prototty::*;

pub struct UiView<V: View<Gws>>(pub V);

const GAME_OFFSET: Coord = Coord::new(1, 3);

const TOP_TEXT_OFFSET: Coord = Coord::new(1, 1);

impl<V: View<Gws>> View<Gws> for UiView<V> {
    fn view<G: ViewGrid>(&mut self, game: &Gws, offset: Coord, depth: i32, grid: &mut G) {
        self.0.view(game, offset + GAME_OFFSET, depth, grid);
    }
}

pub struct DeathView;

impl View<Gws> for DeathView {
    fn view<G: ViewGrid>(&mut self, game: &Gws, offset: Coord, depth: i32, grid: &mut G) {
        DeathGameView.view(game, offset + GAME_OFFSET, depth, grid);
        DefaultRichTextView.view(
            // TODO avoid allocating on each frame
            &RichText::one_line(vec![
                (
                    "YOU DIED",
                    TextInfo::default()
                        .bold()
                        .foreground_colour(rgb24(128, 0, 0)),
                ),
                (
                    " (press any key)",
                    TextInfo::default().bold().foreground_colour(grey24(128)),
                ),
            ]),
            offset + TOP_TEXT_OFFSET,
            depth,
            grid,
        );
    }
}
