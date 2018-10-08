use super::menu;
use alarm::{Alarm, AlarmManager, Mode};
use core::fmt::{self, Write};
use heapless::consts::U40;
use heapless::String;
use il3820::DisplayRibbonLeft;
use rtc::datetime;

#[derive(Debug, Clone)]
pub enum Screen {
    Clock,
    Menu(MenuElt),
    SetClock(EditDateTime),
    ManageAlarms(usize),
    ManageAlarm(ManageAlarm),
}

#[derive(Debug, Copy, Clone)]
pub enum MenuElt {
    Clock,
    SetClock,
    ManageAlarms,
}
impl MenuElt {
    pub fn next(&mut self) {
        use self::MenuElt::*;
        *self = match *self {
            Clock => SetClock,
            SetClock => ManageAlarms,
            ManageAlarms => Clock,
        }
    }
    pub fn prev(&mut self) {
        use self::MenuElt::*;
        *self = match *self {
            Clock => ManageAlarms,
            SetClock => Clock,
            ManageAlarms => SetClock,
        }
    }
    pub fn items(&self) -> &'static [&'static str] {
        &["Main screen", "Set clock", "Manage alarms"]
    }
}

#[derive(Debug, Clone)]
pub struct EditDateTime {
    datetime: datetime::DateTime,
    state: EditDateTimeState,
}
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct ManageAlarm {
    id: usize,
    alarm: Alarm,
    state: ManageAlarmState,
}
impl ManageAlarm {
    pub fn new(manager: &AlarmManager, id: usize) -> Self {
        Self {
            id,
            alarm: manager.alarms[id].clone(),
            state: ManageAlarmState::ToggleEnable,
        }
    }
    pub fn ok(&self) -> Screen {
        use self::ManageAlarmState::*;
        match self.state {
            ToggleEnable => {
                let mut manage = self.clone();
                manage.alarm.is_enable = !manage.alarm.is_enable;
                Screen::ManageAlarm(manage)
            }
            SetTime => Screen::ManageAlarm(self.clone()),
            ToggleOneTime => {
                let mut manage = self.clone();
                manage.alarm.mode.toggle(Mode::ONE_TIME);
                Screen::ManageAlarm(manage)
            }
            Quit => Screen::Clock,
        }
    }
    pub fn next(&mut self) {
        self.state = self.state.next();
    }
    pub fn prev(&mut self) {
        self.state = self.state.prev();
    }
    pub fn render(&self, display: &mut DisplayRibbonLeft) {
        let mut title = String::<U40>::new();
        write!(title, "Edit: {}", self.alarm).unwrap();
        menu::render(
            &title,
            &[
                if self.alarm.is_enable {
                    "Disable"
                } else {
                    "Enable"
                },
                "Set Time",
                if self.alarm.mode.contains(Mode::ONE_TIME) {
                    "Repeat"
                } else {
                    "One time"
                },
                "Save and quit",
            ],
            self.state as i32,
            display,
        );
    }
}
#[derive(Debug, Clone, Copy)]
enum ManageAlarmState {
    ToggleEnable,
    SetTime,
    ToggleOneTime,
    Quit,
}
impl ManageAlarmState {
    pub fn next(&self) -> Self {
        use self::ManageAlarmState::*;
        match *self {
            ToggleEnable => SetTime,
            SetTime => ToggleOneTime,
            ToggleOneTime => Quit,
            Quit => ToggleEnable,
        }
    }
    pub fn prev(&self) -> Self {
        use self::ManageAlarmState::*;
        match *self {
            ToggleEnable => Quit,
            SetTime => ToggleEnable,
            ToggleOneTime => SetTime,
            Quit => ToggleOneTime,
        }
    }
}
