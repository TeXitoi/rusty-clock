use core::fmt;
use rtc::datetime;

#[derive(Clone)]
pub enum Screen {
    Clock,
    Menu(MenuElt),
    SetClock(EditDateTime),
    ManageAlarms(usize),
}

#[derive(Debug, Copy, Clone)]
pub enum MenuElt {
    Clock,
    SetClock,
    ManageAlarms,
}
impl MenuElt {
    pub fn next(&self) -> MenuElt {
        use self::MenuElt::*;
        match *self {
            Clock => SetClock,
            SetClock => ManageAlarms,
            ManageAlarms => Clock,
        }
    }
    pub fn prev(&self) -> MenuElt {
        use self::MenuElt::*;
        match *self {
            Clock => ManageAlarms,
            SetClock => Clock,
            ManageAlarms => SetClock,
        }
    }
    pub fn items(&self) -> &'static [&'static str] {
        &["Main screen", "Set clock", "Manage alarms"]
    }
}
#[derive(Clone)]
pub struct EditDateTime {
    datetime: datetime::DateTime,
    state: EditDateTimeState,
}
#[derive(Clone)]
pub enum EditDateTimeState {
    Year,
    Month,
    Day,
    Hour,
    Min,
}
impl EditDateTime {
    pub fn new(datetime: datetime::DateTime) -> Self {
        Self {
            datetime,
            state: EditDateTimeState::Year,
        }
    }
    pub fn next(&mut self) {
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
    pub fn prev(&mut self) {
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
    pub fn ok(&mut self) -> Option<datetime::DateTime> {
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
