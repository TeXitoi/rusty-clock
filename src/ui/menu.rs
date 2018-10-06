use embedded_graphics::coord::Coord;
use embedded_graphics::fonts::Font8x16;
use embedded_graphics::prelude::*;
use il3820::DisplayRibbonLeft;

const MARGIN_TOP: i32 = 16;
const MARGIN_LEFT: i32 = 4;
const FONT_WIDTH: i32 = 8;
const INTERLINE: i32 = 16;

pub fn render(title: &str, items: &[&str], selected: i32, display: &mut DisplayRibbonLeft) {
    render_str(title, MARGIN_LEFT, MARGIN_TOP, display);
    for (i, &item) in items.iter().enumerate() {
        render_str(
            item,
            MARGIN_LEFT + 3 * FONT_WIDTH,
            MARGIN_TOP + (1 + i as i32) * INTERLINE,
            display,
        );
    }
    render_str(
        ">",
        MARGIN_LEFT + 1 * FONT_WIDTH,
        MARGIN_TOP + (selected + 1) * INTERLINE,
        display,
    );
}

fn render_str(s: &str, x: i32, y: i32, display: &mut DisplayRibbonLeft) {
    display.draw(
        Font8x16::render_str(s)
            .with_stroke(Some(1u8.into()))
            .translate(Coord::new(x, y))
            .into_iter(),
    );
}
