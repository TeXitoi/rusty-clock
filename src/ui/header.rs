use embedded_graphics::coord::Coord;
use embedded_graphics::fonts::Font6x8;
use embedded_graphics::prelude::*;
use il3820::DisplayRibbonLeft;

pub struct Header<'a> {
    display: &'a mut DisplayRibbonLeft,
}
impl<'a> Header<'a> {
    pub fn new(display: &'a mut DisplayRibbonLeft) -> Self {
        Self { display }
    }
    pub fn top_left(&mut self, s: &str) {
        self.render(s, 4, 4);
    }
    pub fn top_center(&mut self, s: &str) {
        let len = s.chars().count() as i32;
        self.render(s, 296 / 2 - 3 * len, 4);
    }
    pub fn top_right(&mut self, s: &str) {
        let len = s.chars().count() as i32;
        self.render(s, 295 - 4 - 6 * len, 4);
    }
    pub fn bottom_left(&mut self, s: &str) {
        self.render(s, 4, 127 - 10);
    }
    pub fn bottom_center(&mut self, s: &str) {
        let len = s.chars().count() as i32;
        self.render(s, 296 / 2 - 3 * len, 127 - 10);
    }
    pub fn bottom_right(&mut self, s: &str) {
        let len = s.chars().count() as i32;
        self.render(s, 295 - 4 - 6 * len, 127 - 10);
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
