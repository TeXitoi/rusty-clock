#[derive(Copy, Clone)]
pub enum Event {
    Pressed,
    Reseased,
    Nothing,
}
#[derive(Copy, Clone)]
enum State {
    High(u8),
    Low(u8),
}
pub struct Button<T> {
    button: T,
    state: State,
}
impl<T: ::embedded_hal::digital::v2::InputPin<Error = core::convert::Infallible>> Button<T> {
    pub fn new(button: T) -> Self {
        Button {
            button,
            state: State::High(0),
        }
    }
    pub fn poll(&mut self) -> Event {
        use self::State::*;
        let value = self.button.is_high().unwrap();
        match &mut self.state {
            High(cnt) => {
                if value {
                    *cnt = 0
                } else {
                    *cnt += 1
                }
            }
            Low(cnt) => {
                if value {
                    *cnt += 1
                } else {
                    *cnt = 0
                }
            }
        }
        match self.state {
            High(cnt) if cnt >= 30 => {
                self.state = Low(0);
                Event::Pressed
            }
            Low(cnt) if cnt >= 30 => {
                self.state = High(0);
                Event::Reseased
            }
            _ => Event::Nothing,
        }
    }
}
