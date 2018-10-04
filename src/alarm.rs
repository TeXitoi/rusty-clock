use bitflags::bitflags;
use rtc::datetime::{DateTime, DayOfWeek};

#[derive(Debug, Clone)]
pub struct AlarmManager {
    pub alarms: [Alarm; 5],
}
impl Default for AlarmManager {
    fn default() -> Self {
        Self {
            alarms: [
                Alarm::default(),
                Alarm::default(),
                Alarm::default(),
                Alarm::default(),
                Alarm::default(),
            ],
        }
    }
}
impl AlarmManager {
    pub fn must_ring(&mut self, datetime: &DateTime) -> bool {
        self.alarms
            .iter_mut()
            .map(|am| am.must_ring(datetime) as u8)
            .sum::<u8>()
            > 0
    }
    pub fn next_ring(&self, dt: &DateTime) -> Option<(DayOfWeek, u8, u8)> {
        let mut min = None;
        for cur in self.alarms.iter().filter_map(|a| a.next_ring(dt)) {
            match min {
                None => min = Some(cur),
                Some(min_val) => {
                    let cmp_value = |(dow, h, m)| {
                        let mut days = (dow as u8 + 7 - dt.day_of_week as u8) % 7;
                        let now_h = time(dt.hour, dt.min);
                        let this_h = time(h, m);
                        if this_h <= now_h {
                            days += 7
                        }
                        days as u32 * 60 * 24 + this_h
                    };
                    if cmp_value(cur) < cmp_value(min_val) {
                        min = Some(cur);
                    }
                }
            }
        }
        min
    }
}

#[derive(Debug, Clone)]
pub struct Alarm {
    pub is_enable: bool,
    hour: u8,
    min: u8,
    pub mode: Mode,
}

bitflags! {
    pub struct Mode: u8 {
        const MONDAY =    0b00000001;
        const TUESDAY =   0b00000010;
        const WEDNESDAY = 0b00000100;
        const THURSDAY =  0b00001000;
        const FRIDAY =    0b00010000;
        const SATURDAY =  0b00100000;
        const SUNDAY =    0b01000000;
        const ONE_TIME =  0b10000000;
    }
}
impl Default for Mode {
    fn default() -> Self {
        Self::all() - Mode::SATURDAY - Mode::SUNDAY
    }
}
impl Mode {
    pub fn contains_dow(&self, dow: DayOfWeek) -> bool {
        use self::DayOfWeek::*;
        match dow {
            Monday => self.contains(Mode::MONDAY),
            Tuesday => self.contains(Mode::TUESDAY),
            Wednesday => self.contains(Mode::WEDNESDAY),
            Thursday => self.contains(Mode::THURSDAY),
            Friday => self.contains(Mode::FRIDAY),
            Saturday => self.contains(Mode::SATURDAY),
            Sunday => self.contains(Mode::SUNDAY),
        }
    }
}

impl Default for Alarm {
    fn default() -> Self {
        Self {
            is_enable: false,
            hour: 12,
            min: 0,
            mode: Mode::default(),
        }
    }
}
impl Alarm {
    pub fn hour(&self) -> u8 {
        self.hour
    }
    pub fn set_hour(&mut self, h: u8) {
        assert!(h < 24);
        self.hour = h;
    }
    pub fn min(&self) -> u8 {
        self.min
    }
    pub fn set_min(&mut self, m: u8) {
        assert!(m < 60);
        self.min = m;
    }
    pub fn must_ring(&mut self, datetime: &DateTime) -> bool {
        if !self.is_enable {
            return false;
        }
        if datetime.sec != 0 || datetime.hour != self.hour || datetime.min != self.min {
            return false;
        }
        if self.mode.contains(Mode::ONE_TIME) {
            self.is_enable = false;
            true
        } else {
            self.mode.contains_dow(datetime.day_of_week)
        }
    }
    pub fn next_ring(&self, datetime: &DateTime) -> Option<(DayOfWeek, u8, u8)> {
        if !self.is_enable || self.mode.is_empty() {
            return None;
        }
        let mut day = if time(self.hour, self.min) <= time(datetime.hour, datetime.min) {
            datetime.day_of_week.next()
        } else {
            datetime.day_of_week
        };
        loop {
            if self.mode.contains_dow(day) {
                return Some((day, self.hour, self.min));
            }
            day = day.next();
        }
    }
}

fn time(hour: u8, min: u8) -> u32 {
    hour as u32 * 60 + min as u32
}
