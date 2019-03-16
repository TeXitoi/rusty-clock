use core::cmp::min;
use core::fmt::Write;
use embedded_graphics::coord::Coord;
use embedded_graphics::fonts::Font8x16;
use embedded_graphics::prelude::*;
use epd_waveshare::epd2in9::Display2in9;
use epd_waveshare::prelude::Color;
use heapless::{consts::*, String};

const MARGIN_TOP: i32 = 16;
const MARGIN_LEFT: i32 = 4;
const FONT_WIDTH: i32 = 8;
const INTERLINE: i32 = 16;

pub fn render(title: &str, mut items: &[&str], mut selected: i32, display: &mut Display2in9) {
    render_str(title, MARGIN_LEFT, MARGIN_TOP, display);

    let len = items.len();
    if len > 5 {
        let page = (selected / 5) as usize;
        selected %= 5;
        items = &items[page * 5..min(page * 5 + 5, len)];

        let mut s = String::<U5>::new();
        write!(s, "{}/{}", page + 1, (len - 1) / 5 + 1).unwrap();
        render_str(
            &s,
            295 - MARGIN_LEFT - 3 * FONT_WIDTH,
            MARGIN_TOP + 5 * INTERLINE,
            display,
        );
    }

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
        MARGIN_LEFT + FONT_WIDTH,
        MARGIN_TOP + (selected + 1) * INTERLINE,
        display,
    );
}

fn render_str(s: &str, x: i32, y: i32, display: &mut Display2in9) {
    display.draw(
        Font8x16::render_str(s)
            .with_stroke(Some(Color::Black))
            .with_fill(Some(Color::White))
            .translate(Coord::new(x, y))
            .into_iter(),
    );
}
