use crate::alarm::{Alarm, AlarmManager};
use crate::datetime;
use core::fmt::{self, Write};
use embedded_graphics::coord::Coord;
use embedded_graphics::fonts::Font8x16;
use embedded_graphics::prelude::*;
use epd_waveshare::epd2in9::Display2in9;
use epd_waveshare::graphics::Display;
use epd_waveshare::prelude::{Color, DisplayRotation};
use heapless::{consts::*, String, Vec};

mod header;
mod menu;
mod seven_segments;
mod state;

#[derive(Debug)]
pub enum Msg {
    DateTime(datetime::DateTime),
    Environment(Environment),
    FailEnvironment,
    ButtonCancel,
    ButtonMinus,
    ButtonPlus,
    ButtonOk,
    AlarmManager(AlarmManager),
}
impl Msg {
    fn is_button(&self) -> bool {
        use self::Msg::*;
        match self {
            ButtonCancel | ButtonMinus | ButtonPlus | ButtonOk => true,
            _ => false,
        }
    }
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
    last_input: u32,
    env: Environment,
    nb_fail_environment: u32,
    alarm_manager: AlarmManager,
    screen: state::Screen,
}

impl Model {
    pub fn init() -> Self {
        Self {
            now: datetime::DateTime::new(0),
            last_input: 0,
            env: Default::default(),
            nb_fail_environment: 0,
            alarm_manager: AlarmManager::default(),
            screen: state::Screen::Clock,
        }
    }
    pub fn update(&mut self, msg: Msg) -> Vec<Cmd, U4> {
        use self::state::Screen::*;
        let mut cmds = Vec::new();

        if msg.is_button() {
            self.update_last_input();
        }

        match msg {
            Msg::DateTime(dt) => {
                self.now = dt;
                if self.screen != state::Screen::Clock
                    && self
                        .now
                        .to_epoch()
                        .map_or(false, |n| n - self.last_input > 10 * 60)
                {
                    self.screen = state::Screen::Clock;
                }
                if self.now.hour == 0 && self.now.min == 0 && self.now.sec == 0 {
                    cmds.push(Cmd::FullUpdate).unwrap();
                }
            }
            Msg::Environment(measurements) => {
                self.env = measurements;
                self.nb_fail_environment = 0;
            }
            Msg::FailEnvironment => self.nb_fail_environment += 1,
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
                    SetClock(mut edit) => {
                        if let Some(dt) = edit.ok() {
                            cmds.push(Cmd::UpdateRtc(dt)).unwrap();
                            Clock
                        } else {
                            SetClock(edit)
                        }
                    }
                    ManageAlarms(i) => ManageAlarm(state::ManageAlarm::new(&self.alarm_manager, i)),
                    ManageAlarm(state) => state.ok(&mut cmds),
                };
                if let Clock = self.screen {
                    cmds.push(Cmd::FullUpdate).unwrap();
                }
            }
            Msg::ButtonCancel => {
                self.screen = match ::core::mem::replace(&mut self.screen, Clock) {
                    Clock => Clock,
                    Menu(mut state) => state.cancel(),
                    SetClock(mut state) => state.cancel(),
                    ManageAlarms(_) => Menu(state::MenuElt::ManageAlarms),
                    ManageAlarm(mut state) => state.cancel(),
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
    pub fn view(&self) -> Display2in9 {
        let mut display = Display2in9::default();
        display.set_rotation(DisplayRotation::Rotate270);
        //let mut display = DisplayRibbonLeft::default();

        self.render_header(&mut display);

        use self::state::Screen::*;
        match &self.screen {
            Clock => self.render_clock(&mut display),
            Menu(elt) => self.render_menu(*elt, &mut display),
            SetClock(datetime) => self.render_set_clock(datetime, &mut display),
            ManageAlarms(i) => self.render_manage_alarms(*i, &mut display),
            ManageAlarm(state) => state.render(&mut display),
        }

        display
    }
    fn update_last_input(&mut self) {
        if let Some(epoch) = self.now.to_epoch() {
            self.last_input = epoch;
        }
    }
    fn render_header(&self, display: &mut Display2in9) {
        let mut header = header::Header::new(display);
        let mut s: String<U128> = String::new();

        write!(
            s,
            "{:4}-{:02}-{:02} {}",
            self.now.year, self.now.month, self.now.day, self.now.day_of_week,
        )
        .unwrap();
        header.top_left(&s);

        match self.alarm_manager.next_ring(&self.now) {
            None => header.bottom_left("No alarm"),
            Some((dow, h, m)) => {
                s.clear();
                write!(s, "Alarm: {} {}:{:02}", dow, h, m).unwrap();
                header.bottom_left(&s);
            }
        }

        s.clear();
        if self.nb_fail_environment != 0 {
            write!(s, "({}s) ", self.nb_fail_environment).unwrap();
        }
        write!(s, "{}hPa", Centi(self.env.pressure as i32)).unwrap();
        header.bottom_right(&s);

        s.clear();
        if self.env.humidity != 0 {
            write!(s, "{:2}%RH  ", self.env.humidity).unwrap();
        }
        write!(s, "{}°C", Centi(i32::from(self.env.temperature))).unwrap();
        header.top_right(&s);
    }
    fn render_clock(&self, display: &mut Display2in9) {
        let mut seven = seven_segments::SevenSegments::new(display, 0, 18);

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
            Font8x16::render_str(&s)
                .with_stroke(Some(Color::Black))
                .with_fill(Some(Color::White))
                .translate(Coord::new(296 - 3 * 8, 17))
                .into_iter(),
        );
    }
    fn render_menu(&self, elt: state::MenuElt, display: &mut Display2in9) {
        menu::render("Menu:", elt.items(), elt as i32, display);
    }
    fn render_set_clock(&self, dt: &state::EditDateTime, display: &mut Display2in9) {
        let mut title: String<U128> = String::new();
        write!(
            title,
            "Edit: {:04}-{:02}-{:02} {:02}:{:02}",
            dt.datetime.year, dt.datetime.month, dt.datetime.day, dt.datetime.hour, dt.datetime.min
        )
        .unwrap();
        menu::render(&title, &[dt.as_edit_str()], 0, display);
    }
    fn render_manage_alarms(&self, i: usize, display: &mut Display2in9) {
        let v: Vec<_, U5> = self
            .alarm_manager
            .alarms
            .iter()
            .map(|a| {
                let mut s = String::<U40>::new();
                write!(s, "{}", a).unwrap();
                s
            })
            .collect();
        let v: Vec<&str, U5> = v.iter().map(|s| s.as_str()).collect();
        menu::render("Select alarm:", &v, i as i32, display);
    }
}

struct Centi(i32);
impl fmt::Display for Centi {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0 < 0 {
            f.write_str("-")?;
        }
        let unit = (self.0 / 100).abs();
        let frac = (self.0 % 100).abs();
        write!(f, "{}.{:02}", unit, frac)
    }
}

#[derive(Clone, Debug)]
pub struct Environment {
    /// unit: Pa
    pub pressure: u32,
    /// unit: c°C
    pub temperature: i16,
    /// unit: %
    pub humidity: u8,
}
impl Default for Environment {
    fn default() -> Self {
        Self {
            pressure: 0,
            temperature: 0,
            humidity: 0,
        }
    }
}

#[cfg(test)]
mod test {
    use super::Centi;
    use std::string::ToString;

    #[test]
    fn centi() {
        assert_eq!(Centi(core::i32::MAX).to_string(), "21474836.47");

        assert_eq!(Centi(1000).to_string(), "10.00");
        assert_eq!(Centi(1001).to_string(), "10.01");
        assert_eq!(Centi(1099).to_string(), "10.99");

        assert_eq!(Centi(0).to_string(), "0.00");
        assert_eq!(Centi(1).to_string(), "0.01");
        assert_eq!(Centi(99).to_string(), "0.99");

        assert_eq!(Centi(-1).to_string(), "-0.01");
        assert_eq!(Centi(-99).to_string(), "-0.99");

        assert_eq!(Centi(-1000).to_string(), "-10.00");
        assert_eq!(Centi(-1001).to_string(), "-10.01");
        assert_eq!(Centi(-1099).to_string(), "-10.99");

        assert_eq!(Centi(core::i32::MIN).to_string(), "-21474836.48");
    }
}
