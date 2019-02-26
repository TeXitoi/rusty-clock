use super::{menu, Cmd};
use crate::alarm::{Alarm, AlarmManager, Mode};
use crate::datetime;
use core::fmt::Write;
use heapless::{consts::*, String, Vec};

//use il3820::DisplayRibbonLeft;
//let mut buffer = Buffer2in9::default();
    //let mut display = Display::new(epd.width(), epd.height(), &mut buffer.buffer);
use epd_waveshare::graphics::Display;



macro_rules! manage_str {
    ( $alarm: ident, $d:ident, $m:ident ) => {
        if $alarm.mode.contains(Mode::$m) {
            concat!("Remove ", stringify!($d))
        } else {
            concat!("Add ", stringify!($d))
        }
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    Clock,
    Menu(MenuElt),
    SetClock(EditDateTime),
    ManageAlarms(usize),
    ManageAlarm(ManageAlarm),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
    pub fn cancel(&mut self) -> Screen {
        Screen::Clock
    }
    pub fn items(self) -> &'static [&'static str] {
        &["Main screen", "Set clock", "Manage alarms"]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditDateTime {
    pub datetime: datetime::DateTime,
    state: EditDateTimeState,
}
#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub fn cancel(&mut self) -> Screen {
        use self::EditDateTimeState::*;
        match self.state {
            Year => return Screen::Menu(MenuElt::SetClock),
            Month => self.state = Year,
            Day => self.state = Month,
            Hour => self.state = Day,
            Min => self.state = Hour,
        }
        Screen::SetClock(self.clone())
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
    pub fn as_edit_str(&self) -> &'static str {
        use self::EditDateTimeState::*;
        match self.state {
            Year => "Set year",
            Month => "Set month",
            Day => "Set day",
            Hour => "Set hour",
            Min => "Set minute",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub fn ok(&self, cmds: &mut Vec<Cmd, U4>) -> Screen {
        self.state.ok(&self, cmds)
    }
    pub fn next(&mut self) {
        self.state = self.state.next(&mut self.alarm);
    }
    pub fn prev(&mut self) {
        self.state = self.state.prev(&mut self.alarm);
    }
    pub fn cancel(&mut self) -> Screen {
        match self.state.cancel() {
            None => Screen::ManageAlarms(self.id),
            Some(state) => Screen::ManageAlarm(Self {
                state,
                ..self.clone()
            }),
        }
    }
    pub fn render(&self, display: &mut Display) {
        self.state.render(&self.alarm, display);
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ManageAlarmState {
    Main(ManageAlarmMainState),
    SetHour,
    SetMin,
    ManageRepeat(ManageAlarmManageRepeatState),
}
impl ManageAlarmState {
    pub fn ok(self, manage: &ManageAlarm, cmds: &mut Vec<Cmd, U4>) -> Screen {
        use self::ManageAlarmState::*;
        match self {
            Main(state) => state.ok(manage, cmds),
            SetHour => {
                let mut manage = manage.clone();
                manage.state = SetMin;
                Screen::ManageAlarm(manage)
            }
            SetMin => {
                let mut manage = manage.clone();
                manage.state = Main(ManageAlarmMainState::SetTime);
                Screen::ManageAlarm(manage)
            }
            ManageRepeat(state) => state.ok(manage),
        }
    }
    pub fn next(self, alarm: &mut Alarm) -> Self {
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
    pub fn prev(self, alarm: &mut Alarm) -> Self {
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
    pub fn cancel(self) -> Option<Self> {
        use self::ManageAlarmState::*;
        match self {
            Main(..) => None,
            SetHour => Some(Main(ManageAlarmMainState::SetTime)),
            SetMin => Some(SetHour),
            ManageRepeat(..) => Some(Main(ManageAlarmMainState::ManageRepeat)),
        }
    }
    pub fn render(self, alarm: &Alarm, display: &mut Display) {
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
                menu::render(&title, &menu, state as i32, display);
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
                menu::render(&title, &menu, state as i32, display);
            }
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ManageAlarmMainState {
    ToggleEnable,
    SetTime,
    ToggleOneTime,
    ManageRepeat,
    Quit,
}
impl ManageAlarmMainState {
    pub fn ok(self, manage: &ManageAlarm, cmds: &mut Vec<Cmd, U4>) -> Screen {
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
            Quit => {
                cmds.push(Cmd::UpdateAlarm(manage.alarm.clone(), manage.id))
                    .unwrap();
                Screen::Clock
            }
        }
    }
    pub fn next(self) -> Self {
        use self::ManageAlarmMainState::*;
        match self {
            ToggleEnable => SetTime,
            SetTime => ToggleOneTime,
            ToggleOneTime => ManageRepeat,
            ManageRepeat => Quit,
            Quit => ToggleEnable,
        }
    }
    pub fn prev(self) -> Self {
        use self::ManageAlarmMainState::*;
        match self {
            ToggleEnable => Quit,
            SetTime => ToggleEnable,
            ToggleOneTime => SetTime,
            ManageRepeat => ToggleOneTime,
            Quit => ManageRepeat,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub fn ok(self, manage: &ManageAlarm) -> Screen {
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
    pub fn next(self) -> Self {
        use self::ManageAlarmManageRepeatState::*;
        match self {
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
    pub fn prev(self) -> Self {
        use self::ManageAlarmManageRepeatState::*;
        match self {
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
