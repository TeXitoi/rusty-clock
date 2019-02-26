use embedded_graphics::coord::Coord;
use embedded_graphics::fonts::Font8x16;
use embedded_graphics::prelude::*;
use epd_waveshare::epd2in9::Display2in9;
use epd_waveshare::prelude::Color;

const MARGIN: i32 = 0;
const FONT_HEIGHT: i32 = 16;
const FONT_WIDTH: i32 = 8;
const BOTTOM_Y: i32 = 128 - MARGIN - FONT_HEIGHT;

pub struct Header<'a> {
    display: &'a mut Display2in9,
}
impl<'a> Header<'a> {
    pub fn new(display: &'a mut Display2in9) -> Self {
        Self { display }
    }
    pub fn top_left(&mut self, s: &str) {
        self.render(s, MARGIN, MARGIN);
    }
    pub fn _top_center(&mut self, s: &str) {
        let len = s.chars().count() as i32;
        self.render(s, 296 / 2 - FONT_WIDTH * len / 2, MARGIN);
    }
    pub fn top_right(&mut self, s: &str) {
        let len = s.chars().count() as i32;
        self.render(s, 295 - MARGIN - FONT_WIDTH * len, MARGIN);
    }
    pub fn bottom_left(&mut self, s: &str) {
        self.render(s, MARGIN, BOTTOM_Y);
    }
    pub fn _bottom_center(&mut self, s: &str) {
        let len = s.chars().count() as i32;
        self.render(s, 296 / 2 - FONT_WIDTH * len / 2, BOTTOM_Y);
    }
    pub fn bottom_right(&mut self, s: &str) {
        let len = s.chars().count() as i32;
        self.render(s, 295 - MARGIN - FONT_WIDTH * len, BOTTOM_Y);
    }
    fn render(&mut self, s: &str, x: i32, y: i32) {
        self.display.draw(
            Font8x16::render_str(s)
                .with_stroke(Some(Color::Black))
                .translate(Coord::new(x, y))
                .into_iter(),
        );
    }
}
