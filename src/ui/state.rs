use super::menu;
use alarm::{Alarm, AlarmManager, Mode};
use core::fmt::{self, Write};
use heapless::consts::U40;
use heapless::String;
use il3820::DisplayRibbonLeft;
use rtc::datetime;

macro_rules! manage_str {
    ( $alarm: ident, $d:ident, $m:ident ) => {
        if $alarm.mode.contains(Mode::$m) {
            concat!("Remove ", stringify!($d))
        } else {
            concat!("Add ", stringify!($d))
        }
    };
}

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
            state: ManageAlarmState::Main(ManageAlarmMainState::ToggleEnable),
        }
    }
    pub fn ok(&self) -> Screen {
        self.state.ok(&self)
    }
    pub fn next(&mut self) {
        self.state = self.state.next(&mut self.alarm);
    }
    pub fn prev(&mut self) {
        self.state = self.state.prev(&mut self.alarm);
    }
    pub fn render(&self, display: &mut DisplayRibbonLeft) {
        self.state.render(&self.alarm, display);
    }
}
#[derive(Debug, Clone, Copy)]
enum ManageAlarmState {
    Main(ManageAlarmMainState),
    SetHour,
    SetMin,
    ManageRepeat(ManageAlarmManageRepeatState),
}
impl ManageAlarmState {
    pub fn ok(&self, manage: &ManageAlarm) -> Screen {
        use self::ManageAlarmState::*;
        match self {
            Main(state) => state.ok(manage),
            SetHour => {
                let mut manage = manage.clone();
                manage.state = SetMin;
                Screen::ManageAlarm(manage)
            },
            SetMin => {
                let mut manage = manage.clone();
                manage.state = Main(ManageAlarmMainState::SetTime);
                Screen::ManageAlarm(manage)
            },
            ManageRepeat(state) => state.ok(manage),
        }
    }
    pub fn next(&self, alarm: &mut Alarm) -> Self {
        use self::ManageAlarmState::*;
        match self {
            Main(state) => Main(state.next()),
            SetHour => {
                let hour = alarm.hour();
                alarm.set_hour((hour + 1) % 24);
                SetHour
            }
            SetMin => {
                let min = alarm.min();
                alarm.set_min((min + 5) % 60);
                SetMin
            }
            ManageRepeat(state) => ManageRepeat(state.next()),
        }
    }
    pub fn prev(&self, alarm: &mut Alarm) -> Self {
        use self::ManageAlarmState::*;
        match self {
            Main(state) => Main(state.prev()),
            SetHour => {
                let hour = alarm.hour();
                alarm.set_hour((hour + 24 - 1) % 24);
                SetHour
            }
            SetMin => {
                let min = alarm.min();
                alarm.set_min((min + 60 - 5) % 60);
                SetMin
            }
            ManageRepeat(state) => ManageRepeat(state.prev()),
        }
    }
    pub fn render(&self, alarm: &Alarm, display: &mut DisplayRibbonLeft) {
        use self::ManageAlarmState::*;

        let mut title = String::<U40>::new();
        write!(title, "Edit: {}", alarm).unwrap();
        match self {
            Main(state) => {
                let menu = [
                    if alarm.is_enable { "Disable" } else { "Enable" },
                    "Set Time",
                    if alarm.mode.contains(Mode::ONE_TIME) {
                        "Repeat"
                    } else {
                        "One time"
                    },
                    "Manage repeat",
                    "Save and quit",
                ];
                menu::render(&title, &menu, *state as i32, display);
            }
            SetHour => menu::render(&title, &["Set hour"], 0, display),
            SetMin => menu::render(&title, &["Set minute"], 0, display),
            ManageRepeat(state) => {
                let menu = [
                    manage_str!(alarm, Monday, MONDAY),
                    manage_str!(alarm, Tuesday, TUESDAY),
                    manage_str!(alarm, Wednesday, WEDNESDAY),
                    manage_str!(alarm, Thursday, THURSDAY),
                    manage_str!(alarm, Friday, FRIDAY),
                    manage_str!(alarm, Saturday, SATURDAY),
                    manage_str!(alarm, Sunday, SUNDAY),
                    "Back",
                ];
                menu::render(&title, &menu, *state as i32, display);
            }
        }
    }
}
#[derive(Debug, Clone, Copy)]
enum ManageAlarmMainState {
    ToggleEnable,
    SetTime,
    ToggleOneTime,
    ManageRepeat,
    Quit,
}
impl ManageAlarmMainState {
    pub fn ok(&self, manage: &ManageAlarm) -> Screen {
        use self::ManageAlarmMainState::*;
        match self {
            ToggleEnable => {
                let mut manage = manage.clone();
                manage.alarm.is_enable = !manage.alarm.is_enable;
                Screen::ManageAlarm(manage)
            }
            SetTime => {
                let mut manage = manage.clone();
                manage.state = ManageAlarmState::SetHour;
                Screen::ManageAlarm(manage)
            }
            ToggleOneTime => {
                let mut manage = manage.clone();
                manage.alarm.mode.toggle(Mode::ONE_TIME);
                Screen::ManageAlarm(manage)
            }
            ManageRepeat => {
                let mut manage = manage.clone();
                use self::ManageAlarmManageRepeatState::Monday;
                manage.state = ManageAlarmState::ManageRepeat(Monday);
                Screen::ManageAlarm(manage)
            }
            Quit => Screen::Clock,
        }
    }
    pub fn next(&self) -> Self {
        use self::ManageAlarmMainState::*;
        match *self {
            ToggleEnable => SetTime,
            SetTime => ToggleOneTime,
            ToggleOneTime => ManageRepeat,
            ManageRepeat => Quit,
            Quit => ToggleEnable,
        }
    }
    pub fn prev(&self) -> Self {
        use self::ManageAlarmMainState::*;
        match *self {
            ToggleEnable => Quit,
            SetTime => ToggleEnable,
            ToggleOneTime => SetTime,
            ManageRepeat => ToggleOneTime,
            Quit => ManageRepeat,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ManageAlarmManageRepeatState {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
    Quit,
}
impl ManageAlarmManageRepeatState {
    pub fn ok(&self, manage: &ManageAlarm) -> Screen {
        use self::ManageAlarmManageRepeatState::*;
        let toggle = |d| {
            let mut manage = manage.clone();
            manage.alarm.mode.toggle(d);
            Screen::ManageAlarm(manage)
        };
        match self {
            Monday => toggle(Mode::MONDAY),
            Tuesday => toggle(Mode::TUESDAY),
            Wednesday => toggle(Mode::WEDNESDAY),
            Thursday => toggle(Mode::THURSDAY),
            Friday => toggle(Mode::FRIDAY),
            Saturday => toggle(Mode::SATURDAY),
            Sunday => toggle(Mode::SUNDAY),
            Quit => {
                let mut manage = manage.clone();
                manage.state = ManageAlarmState::Main(ManageAlarmMainState::ManageRepeat);
                Screen::ManageAlarm(manage)
            }
        }
    }
    pub fn next(&self) -> Self {
        use self::ManageAlarmManageRepeatState::*;
        match *self {
            Monday => Tuesday,
            Tuesday => Wednesday,
            Wednesday => Thursday,
            Thursday => Friday,
            Friday => Saturday,
            Saturday => Sunday,
            Sunday => Quit,
            Quit => Monday,
        }
    }
    pub fn prev(&self) -> Self {
        use self::ManageAlarmManageRepeatState::*;
        match *self {
            Monday => Quit,
            Tuesday => Monday,
            Wednesday => Tuesday,
            Thursday => Wednesday,
            Friday => Thursday,
            Saturday => Friday,
            Sunday => Saturday,
            Quit => Sunday,
        }
    }
}
