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
    SetClock(datetime::DateTime),
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
            Msg::ButtonOk => match self.screen {
                Clock => self.screen = Menu(MenuElt::Clock),
                Menu(MenuElt::Clock) => self.screen = Clock,
                Menu(MenuElt::SetClock) => self.screen = SetClock(self.now.clone()),
                SetClock(_) => self.screen = Clock,
            },
            Msg::ButtonPlus => match &mut self.screen {
                Menu(elt) => *elt = elt.next(),
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
