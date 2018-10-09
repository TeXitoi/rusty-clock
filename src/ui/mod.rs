use alarm::{Alarm, AlarmManager};
use core::fmt::{self, Write};
use embedded_graphics::coord::Coord;
use embedded_graphics::fonts::{Font6x8, Font8x16};
use embedded_graphics::prelude::*;
use embedded_hal::blocking::i2c::WriteRead;
use heapless::{consts::*, String, Vec};
use il3820::DisplayRibbonLeft;
use rtc::datetime;

mod header;
mod menu;
mod seven_segments;
mod state;

#[derive(Debug)]
pub enum Msg {
    DateTime(datetime::DateTime),
    Environment(::bme280::Measurements<<::I2C as WriteRead>::Error>),
    ButtonMinus,
    ButtonOk,
    ButtonPlus,
    AlarmManager(AlarmManager),
}

#[derive(Debug)]
pub enum Cmd {
    UpdateRtc(datetime::DateTime),
    UpdateAlarm(Alarm, usize),
    FullUpdate,
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
    screen: state::Screen,
}

impl Model {
    pub fn init() -> Self {
        Self {
            now: datetime::DateTime::new(0),
            pressure: 0,
            temperature: 0,
            humidity: 0,
            alarm_manager: AlarmManager::default(),
            screen: state::Screen::Clock,
        }
    }
    pub fn update(&mut self, msg: Msg) -> Vec<Cmd, U4> {
        use self::state::Screen::*;
        let mut cmds = Vec::new();

        match msg {
            Msg::DateTime(dt) => {
                self.now = dt;
                if self.now.hour == 0 && self.now.min == 0 && self.now.sec == 0 {
                    cmds.push(Cmd::FullUpdate).unwrap();
                }
            }
            Msg::Environment(measurements) => {
                self.pressure = measurements.pressure as u32;
                self.temperature = (measurements.temperature * 100.) as i16;
                self.humidity = measurements.humidity as u8;
            }
            Msg::AlarmManager(am) => self.alarm_manager = am,
            Msg::ButtonOk => {
                use self::state::{EditDateTime, MenuElt};
                self.screen = match ::core::mem::replace(&mut self.screen, Clock) {
                    Clock => Menu(MenuElt::Clock),
                    Menu(MenuElt::Clock) => Clock,
                    Menu(MenuElt::SetClock) => {
                        let mut dt = self.now.clone();
                        dt.sec = 0;
                        SetClock(EditDateTime::new(dt))
                    }
                    Menu(MenuElt::ManageAlarms) => ManageAlarms(0),
                    SetClock(mut edit) => if let Some(dt) = edit.ok() {
                        cmds.push(Cmd::UpdateRtc(dt)).unwrap();
                        Clock
                    } else {
                        SetClock(edit)
                    },
                    ManageAlarms(i) => ManageAlarm(state::ManageAlarm::new(&self.alarm_manager, i)),
                    ManageAlarm(state) => state.ok(&mut cmds),
                };
                if let Clock = self.screen {
                    cmds.push(Cmd::FullUpdate).unwrap();
                }
            }
            Msg::ButtonPlus => match &mut self.screen {
                Clock => {}
                Menu(state) => state.next(),
                SetClock(state) => state.next(),
                ManageAlarms(i) => *i = (*i + 1) % self.alarm_manager.alarms.len(),
                ManageAlarm(state) => state.next(),
            },
            Msg::ButtonMinus => match &mut self.screen {
                Clock => {}
                Menu(state) => state.prev(),
                SetClock(state) => state.prev(),
                ManageAlarms(i) => {
                    let len = self.alarm_manager.alarms.len();
                    *i = (*i + len - 1) % len;
                }
                ManageAlarm(state) => state.prev(),
            },
        }
        cmds
    }
    pub fn view(&self) -> DisplayRibbonLeft {
        let mut display = DisplayRibbonLeft::default();

        self.render_header(&mut display);

        use self::state::Screen::*;
        match &self.screen {
            Clock => self.render_clock(&mut display),
            Menu(elt) => self.render_menu(elt, &mut display),
            SetClock(datetime) => self.render_set_clock(datetime, &mut display),
            ManageAlarms(i) => self.render_manage_alarms(*i, &mut display),
            ManageAlarm(state) => state.render(&mut display),
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
    fn render_menu(&self, elt: &state::MenuElt, display: &mut DisplayRibbonLeft) {
        menu::render("Menu:", elt.items(), *elt as i32, display);
    }
    fn render_set_clock(&self, datetime: &state::EditDateTime, display: &mut DisplayRibbonLeft) {
        let mut s: String<U128> = String::new();
        write!(s, "Set clock: {}", datetime).unwrap();
        display.draw(
            Font8x16::render_str(&s)
                .with_stroke(Some(1u8.into()))
                .translate(Coord::new(12, 44))
                .into_iter(),
        );
    }
    fn render_manage_alarms(&self, i: usize, display: &mut DisplayRibbonLeft) {
        let v: Vec<_, U5> = self
            .alarm_manager
            .alarms
            .iter()
            .map(|a| {
                let mut s = String::<U40>::new();
                write!(s, "{}", a).unwrap();
                s
            }).collect();
        let v: Vec<&str, U5> = v.iter().map(|s| s.as_str()).collect();
        menu::render("Select alarm:", &v, i as i32, display);
    }
}

struct Centi(i32);
impl fmt::Display for Centi {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{:02}", self.0 / 100, self.0 % 100)
    }
}
