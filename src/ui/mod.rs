use core::fmt::{self, Write};
use embedded_graphics::coord::Coord;
use embedded_graphics::fonts::{Font6x8, Font8x16};
use embedded_graphics::prelude::*;
use embedded_hal::blocking::i2c::WriteRead;
use heapless::{consts::*, String, Vec};
use il3820::DisplayRibbonLeft;
use rtc::datetime;
use alarm::AlarmManager;

mod header;
mod seven_segments;

pub enum Msg {
    DateTime(datetime::DateTime),
    Environment(::bme280::Measurements<<::I2C as WriteRead>::Error>),
    ButtonMinus,
    ButtonOk,
    ButtonPlus,
    AlarmManager(AlarmManager),
}

pub enum Cmd {
    UpdateRtc(datetime::DateTime),
}

#[derive(Clone)]
pub struct Model {
    now: datetime::DateTime,
    /// unit: Pa
    pressure: u32,
    /// unit: c°C
    temperature: i16,
    /// unit: %
    humidity: u8,
    alarm_manager: AlarmManager,
    screen: Screen,
}

impl Model {
    pub fn init() -> Self {
        Self {
            now: datetime::DateTime::new(0),
            pressure: 0,
            temperature: 0,
            humidity: 0,
            alarm_manager: AlarmManager::default(),
            screen: Screen::Clock,
        }
    }
    pub fn update(&mut self, msg: Msg) -> Vec<Cmd, U4> {
        use self::Screen::*;
        let mut cmds = Vec::new();

        match msg {
            Msg::DateTime(datetime) => self.now = datetime,
            Msg::Environment(measurements) => {
                self.pressure = measurements.pressure as u32;
                self.temperature = (measurements.temperature * 100.) as i16;
                self.humidity = measurements.humidity as u8;
            }
            Msg::AlarmManager(am) => self.alarm_manager = am,
            Msg::ButtonOk => {
                self.screen = match ::core::mem::replace(&mut self.screen, Clock) {
                    Clock => Menu(MenuElt::Clock),
                    Menu(MenuElt::Clock) => Clock,
                    Menu(MenuElt::SetClock) => {
                        let mut dt = self.now.clone();
                        dt.sec = 0;
                        SetClock(EditDateTime::new(dt))
                    }
                    SetClock(mut edit) => if let Some(dt) = edit.ok() {
                        if let Err(_) = cmds.push(Cmd::UpdateRtc(dt)) {
                            panic!("cmds too small");
                        }
                        Clock
                    } else {
                        SetClock(edit)
                    },
                }
            }
            Msg::ButtonPlus => match &mut self.screen {
                Menu(elt) => *elt = elt.next(),
                SetClock(edit) => edit.next(),
                _ => {}
            },
            Msg::ButtonMinus => match &mut self.screen {
                Menu(elt) => *elt = elt.prev(),
                SetClock(edit) => edit.prev(),
                _ => {}
            },
        }
        cmds
    }
    pub fn view(&self) -> DisplayRibbonLeft {
        let mut display = DisplayRibbonLeft::default();

        self.render_header(&mut display);

        use self::Screen::*;
        match &self.screen {
            Clock => self.render_clock(&mut display),
            Menu(elt) => self.render_menu(elt, &mut display),
            SetClock(datetime) => self.render_set_clock(datetime, &mut display),
        }

        display
    }
    fn render_header(&self, display: &mut DisplayRibbonLeft) {
        let mut header = header::Header::new(display);
        let mut s: String<U128> = String::new();

        write!(
            s,
            "{:4}-{:02}-{:02} {}",
            self.now.year, self.now.month, self.now.day, self.now.day_of_week,
        ).unwrap();
        header.top_left(&s);

        match self.alarm_manager.next_ring(&self.now) {
            None => header.top_right("No alarm"),
            Some((dow, h, m)) => {
                s.clear();
                write!(s, "Alarm: {} {}:{:02}", dow, h, m);
                header.top_right(&s);
            }
        }

        s.clear();
        write!(s, "{}°C", Centi(self.temperature as i32)).unwrap();
        header.bottom_left(&s);

        s.clear();
        write!(s, "{}hPa", Centi(self.pressure as i32),).unwrap();
        header.bottom_right(&s);

        if self.humidity != 0 {
            s.clear();
            write!(s, "{:2}%RH", self.humidity).unwrap();
            header.bottom_center(&s);
        }
    }
    fn render_clock(&self, display: &mut DisplayRibbonLeft) {
        let mut seven = seven_segments::SevenSegments::new(display, 4, 18);

        if self.now.hour >= 10 {
            seven.digit(self.now.hour / 10);
        } else {
            seven.digit_space();
        }
        seven.digit(self.now.hour % 10);
        if self.now.sec % 2 == 0 {
            seven.colon();
        } else {
            seven.colon_space();
        }
        seven.digit(self.now.min / 10);
        seven.digit(self.now.min % 10);

        let display = seven.into_display();
        let mut s: String<U4> = String::new();
        write!(s, ":{:02}", self.now.sec).unwrap();
        display.draw(
            Font6x8::render_str(&s)
                .with_stroke(Some(1u8.into()))
                .translate(Coord::new(273, 18))
                .into_iter(),
        );
    }
    fn render_menu(&self, elt: &MenuElt, display: &mut DisplayRibbonLeft) {
        let mut s: String<U128> = String::new();
        write!(s, "Menu: {}", elt).unwrap();
        display.draw(
            Font8x16::render_str(&s)
                .with_stroke(Some(1u8.into()))
                .translate(Coord::new(12, 44))
                .into_iter(),
        );
    }
    fn render_set_clock(&self, datetime: &EditDateTime, display: &mut DisplayRibbonLeft) {
        let mut s: String<U128> = String::new();
        write!(s, "Set clock: {}", datetime).unwrap();
        display.draw(
            Font8x16::render_str(&s)
                .with_stroke(Some(1u8.into()))
                .translate(Coord::new(12, 44))
                .into_iter(),
        );
    }
}

struct Centi(i32);
impl fmt::Display for Centi {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{:02}", self.0 / 100, self.0 % 100)
    }
}

#[derive(Clone)]
enum Screen {
    Clock,
    Menu(MenuElt),
    SetClock(EditDateTime),
}
#[derive(Clone)]
enum MenuElt {
    Clock,
    SetClock,
}
impl MenuElt {
    fn next(&self) -> MenuElt {
        use self::MenuElt::*;
        match *self {
            Clock => SetClock,
            SetClock => Clock,
        }
    }
    fn prev(&self) -> MenuElt {
        use self::MenuElt::*;
        match *self {
            Clock => SetClock,
            SetClock => Clock,
        }
    }
}
impl fmt::Display for MenuElt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MenuElt::Clock => f.write_str("clock"),
            MenuElt::SetClock => f.write_str("set clock"),
        }
    }
}
#[derive(Clone)]
struct EditDateTime {
    datetime: datetime::DateTime,
    state: EditDateTimeState,
}
#[derive(Clone)]
enum EditDateTimeState {
    Year,
    Month,
    Day,
    Hour,
    Min,
}
impl EditDateTime {
    fn new(datetime: datetime::DateTime) -> Self {
        Self {
            datetime,
            state: EditDateTimeState::Year,
        }
    }
    fn next(&mut self) {
        use self::EditDateTimeState::*;
        match self.state {
            Year => {
                self.datetime.year += 1;
                if self.datetime.year > 2105 {
                    self.datetime.year = 1970;
                }
            }
            Month => self.datetime.month = self.datetime.month % 12 + 1,
            Day => self.datetime.day = self.datetime.day % 31 + 1,
            Hour => self.datetime.hour = (self.datetime.hour + 1) % 24,
            Min => self.datetime.min = (self.datetime.min + 1) % 60,
        }
    }
    fn prev(&mut self) {
        use self::EditDateTimeState::*;
        match self.state {
            Year => {
                self.datetime.year -= 1;
                if self.datetime.year < 1970 {
                    self.datetime.year = 2105;
                }
            }
            Month => self.datetime.month = (self.datetime.month + 12 - 2) % 12 + 1,
            Day => self.datetime.day = (self.datetime.day + 31 - 2) % 31 + 1,
            Hour => self.datetime.hour = (self.datetime.hour + 24 - 1) % 24,
            Min => self.datetime.min = (self.datetime.min + 60 - 1) % 60,
        }
    }
    fn ok(&mut self) -> Option<datetime::DateTime> {
        use self::EditDateTimeState::*;
        match self.state {
            Year => self.state = Month,
            Month => self.state = Day,
            Day => self.state = Hour,
            Hour => self.state = Min,
            Min => return Some(self.datetime.clone()),
        }
        None
    }
}
impl fmt::Display for EditDateTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::EditDateTimeState::*;
        match self.state {
            Year => write!(f, "year: {}", self.datetime.year),
            Month => write!(f, "month: {}", self.datetime.month),
            Day => write!(f, "day: {}", self.datetime.day),
            Hour => write!(f, "hour: {}", self.datetime.hour),
            Min => write!(f, "min: {}", self.datetime.min),
        }
    }
}
