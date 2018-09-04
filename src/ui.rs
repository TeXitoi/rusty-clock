use core::fmt::{self, Write};
use embedded_hal::blocking::i2c::WriteRead;
use heapless;
use rtc::datetime;

pub enum Msg {
    DateTime(datetime::DateTime),
    Environment(::bme280::Measurements<<::I2C as WriteRead>::Error>),
    ButtonOk,
    ButtonPlus,
}

struct Centi(i32);
impl fmt::Display for Centi {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{:02}", self.0 / 100, self.0 % 100)
    }
}

#[derive(Debug, Clone)]
enum Screen {
    Clock,
    Menu(MenuElt),
    SetClock(EditDateTime),
}
#[derive(Debug, Clone)]
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
}
#[derive(Debug, Clone)]
struct EditDateTime {
    datetime: datetime::DateTime,
    state: EditDateTimeState,
}
#[derive(Debug, Clone)]
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
                if self.datetime.year > 2100 {
                    self.datetime.year = 2000;
                }
            }
            Month => self.datetime.month = self.datetime.month % 12 + 1,
            Day => self.datetime.day = self.datetime.day % 31 + 1,
            Hour => self.datetime.hour = (self.datetime.hour + 1) % 24,
            Min => self.datetime.min = (self.datetime.min + 1) % 60,
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

#[derive(Debug, Clone)]
pub struct Model {
    now: datetime::DateTime,
    /// unit: Pa
    pressure: u32,
    /// unit: c°C
    temperature: i16,
    /// unit: %
    humidity: u8,
    screen: Screen,
}

impl Model {
    pub fn init() -> Self {
        Self {
            now: datetime::DateTime::new(0),
            pressure: 0,
            temperature: 0,
            humidity: 0,
            screen: Screen::Clock,
        }
    }
    pub fn update(&mut self, msg: Msg) {
        use self::Screen::*;
        match msg {
            Msg::DateTime(datetime) => self.now = datetime,
            Msg::Environment(measurements) => {
                self.pressure = measurements.pressure as u32;
                self.temperature = (measurements.temperature * 100.) as i16;
                self.humidity = measurements.humidity as u8;
            }
            Msg::ButtonOk => {
                self.screen = match ::core::mem::replace(&mut self.screen, Clock) {
                    Clock => Menu(MenuElt::Clock),
                    Menu(MenuElt::Clock) => Clock,
                    Menu(MenuElt::SetClock) => {
                        let mut dt = self.now.clone();
                        dt.sec = 0;
                        SetClock(EditDateTime::new(dt))
                    }
                    SetClock(mut edit) => if let Some(_dt) = edit.ok() {
                        // TODO: change now
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
        }
        ::request_render();
    }
    pub fn view(&self, r: &mut ::EXTI1::Resources) -> fmt::Result {
        use self::Screen::*;
        let mut s = heapless::String::<heapless::consts::U128>::new();

        writeln!(s)?;
        writeln!(s, "{}", self.now)?;

        match &self.screen {
            Clock => {
                writeln!(s, "Temperature = {}°C", Centi(self.temperature as i32))?;
                writeln!(s, "Pressure = {}hPa", Centi(self.pressure as i32))?;
                if self.humidity != 0 {
                    writeln!(s, "humidity = {}%", self.humidity)?;
                }
            }
            Menu(elt) => writeln!(s, "Menu: {:?}", elt)?,
            SetClock(datetime) => writeln!(s, "Set clock: {}", datetime)?,
        }

        r.DISPLAY.write_str(&s)
    }
}
