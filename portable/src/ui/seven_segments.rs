use embedded_graphics::coord::Coord;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rect;
use epd_waveshare::epd2in9::Display2in9;
use epd_waveshare::prelude::Color;

pub struct SevenSegments<'a> {
    display: &'a mut Display2in9,
    width: i32,
    height: i32,
    thickness: i32,
    space: i32,
    x: i32,
    y: i32,
}
impl<'a> SevenSegments<'a> {
    pub fn new(display: &'a mut Display2in9, x: i32, y: i32) -> Self {
        Self {
            display,
            width: 52,
            height: 90,
            thickness: 12,
            space: 13,
            x,
            y,
        }
    }
    pub fn into_display(self) -> &'a mut Display2in9 {
        self.display
    }
    pub fn colon_space(&mut self) {
        self.x += self.thickness + self.space;
    }
    pub fn colon(&mut self) {
        let t = self.thickness;
        let intern = (self.height - 3 * t) / 2;
        let h1 = t + intern / 2 - t / 2;
        let h2 = self.height - t - intern / 2 - t / 2;
        self.draw_rect(0, h1, t - 1, h1 + t - 1);
        self.draw_rect(0, h2, t - 1, h2 + t - 1);

        self.colon_space();
    }
    pub fn digit_space(&mut self) {
        self.x += self.width + self.space;
    }
    pub fn digit(&mut self, c: u8) {
        fn s(s: u8) -> u8 {
            1 << s
        }
        let segments = match c {
            0 => s(0) | s(1) | s(2) | s(4) | s(5) | s(6),
            1 => s(2) | s(5),
            2 => s(0) | s(2) | s(3) | s(4) | s(6),
            3 => s(0) | s(2) | s(3) | s(5) | s(6),
            4 => s(1) | s(2) | s(3) | s(5),
            5 => s(0) | s(1) | s(3) | s(5) | s(6),
            6 => s(0) | s(1) | s(3) | s(4) | s(5) | s(6),
            7 => s(0) | s(2) | s(5),
            8 => s(0) | s(1) | s(2) | s(3) | s(4) | s(5) | s(6),
            9 => s(0) | s(1) | s(2) | s(3) | s(5) | s(6),
            _ => 0,
        };

        let (h, w, t) = (self.height, self.width, self.thickness);
        let h2 = (h - 3 * t) / 2 + t;
        if segments & 1 != 0 {
            self.draw_rect(0, 0, w - 1, t - 1);
        }
        if segments & (1 << 1) != 0 {
            self.draw_rect(0, 0, t - 1, h2 + t - 1);
        }
        if segments & (1 << 2) != 0 {
            self.draw_rect(w - t, 0, w - 1, h2 + t - 1);
        }
        if segments & (1 << 3) != 0 {
            self.draw_rect(t, h2, w - t - 1, h2 + t - 1);
        }
        if segments & (1 << 4) != 0 {
            self.draw_rect(0, h2, t - 1, h - 1);
        }
        if segments & (1 << 5) != 0 {
            self.draw_rect(w - t, h2, w - 1, h - 1);
        }
        if segments & (1 << 6) != 0 {
            self.draw_rect(0, h - t, w - 1, h - 1);
        }

        self.digit_space();
    }
    fn draw_rect(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        self.display.draw(
            Rect::new(Coord::new(x1, y1), Coord::new(x2, y2))
                .with_fill(Some(Color::Black))
                .translate(Coord::new(self.x, self.y))
                .into_iter(),
        );
    }
}
