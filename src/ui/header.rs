use embedded_graphics::coord::Coord;
use embedded_graphics::fonts::Font6x8;
use embedded_graphics::prelude::*;
use il3820::DisplayRibbonLeft;

const MARGIN: i32 = 4;
const FONT_HEIGHT: i32 = 8;
const FONT_WIDTH: i32 = 6;
const BOTTOM_Y: i32 = 128 - MARGIN - FONT_HEIGHT;

pub struct Header<'a> {
    display: &'a mut DisplayRibbonLeft,
}
impl<'a> Header<'a> {
    pub fn new(display: &'a mut DisplayRibbonLeft) -> Self {
        Self { display }
    }
    pub fn top_left(&mut self, s: &str) {
        self.render(s, MARGIN, MARGIN);
    }
    pub fn top_center(&mut self, s: &str) {
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
    pub fn bottom_center(&mut self, s: &str) {
        let len = s.chars().count() as i32;
        self.render(s, 296 / 2 - FONT_WIDTH * len / 2, BOTTOM_Y);
    }
    pub fn bottom_right(&mut self, s: &str) {
        let len = s.chars().count() as i32;
        self.render(s, 295 - MARGIN - FONT_WIDTH * len, BOTTOM_Y);
    }
    fn render(&mut self, s: &str, x: i32, y: i32) {
        self.display.draw(
            Font6x8::render_str(s)
                .with_stroke(Some(1u8.into()))
                .translate(Coord::new(x, y))
                .into_iter(),
        );
    }
}
