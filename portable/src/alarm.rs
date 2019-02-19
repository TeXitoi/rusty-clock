use crate::datetime::{DateTime, DayOfWeek};
use bitflags::bitflags;
use core::fmt;

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
                        if dow == dt.day_of_week && this_h <= now_h {
                            days += 7
                        }
                        u32::from(days) * 60 * 24 + this_h
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alarm {
    pub is_enable: bool,
    hour: u8,
    min: u8,
    pub mode: Mode,
}

bitflags! {
    pub struct Mode: u8 {
        const MONDAY =    0b0000_0001;
        const TUESDAY =   0b0000_0010;
        const WEDNESDAY = 0b0000_0100;
        const THURSDAY =  0b0000_1000;
        const FRIDAY =    0b0001_0000;
        const SATURDAY =  0b0010_0000;
        const SUNDAY =    0b0100_0000;
        const ONE_TIME =  0b1000_0000;
    }
}
impl Default for Mode {
    fn default() -> Self {
        Self::all() - Mode::SATURDAY - Mode::SUNDAY
    }
}
impl Mode {
    pub fn contains_dow(self, dow: DayOfWeek) -> bool {
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
        if self.mode.contains(Mode::ONE_TIME) {
            return Some((day, self.hour, self.min));
        }
        loop {
            if self.mode.contains_dow(day) {
                return Some((day, self.hour, self.min));
            }
            day = day.next();
        }
    }
}
impl fmt::Display for Alarm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_enable {
            f.write_str("On  ")?;
        } else {
            f.write_str("Off ")?;
        }
        write!(f, "{:02}:{:02}", self.hour, self.min)?;
        if self.mode.contains(Mode::ONE_TIME) {
            f.write_str(" one time")?;
        } else if self.mode.is_empty() {
            f.write_str(" never")?;
        } else {
            for &(dow, s) in VEC_DAY_OF_WEEK_SHORT_NAME.iter() {
                if self.mode.contains(dow) {
                    f.write_str(s)?;
                }
            }
        }
        Ok(())
    }
}

static VEC_DAY_OF_WEEK_SHORT_NAME: [(Mode, &str); 7] = [
    (Mode::MONDAY, " Mo"),
    (Mode::TUESDAY, " Tu"),
    (Mode::WEDNESDAY, " We"),
    (Mode::THURSDAY, " Th"),
    (Mode::FRIDAY, " Fr"),
    (Mode::SATURDAY, " Sa"),
    (Mode::SUNDAY, " Su"),
];
fn time(hour: u8, min: u8) -> u32 {
    u32::from(hour) * 60 + u32::from(min)
}

#[cfg(test)]
mod test {
    use super::*;

    fn default_alarm_manager() -> AlarmManager {
        let mut alarm_manager = AlarmManager::default();
        alarm_manager.alarms[0].is_enable = true;
        alarm_manager.alarms[0].set_hour(7);
        alarm_manager.alarms[0].set_min(25);
        alarm_manager.alarms[0].mode = Mode::MONDAY | Mode::TUESDAY | Mode::THURSDAY | Mode::FRIDAY;
        alarm_manager.alarms[1].is_enable = true;
        alarm_manager.alarms[1].set_hour(8);
        alarm_manager.alarms[1].set_min(15);
        alarm_manager.alarms[1].mode = Mode::WEDNESDAY;
        alarm_manager
    }

    #[test]
    fn test_next_ring() {
        let alarm_manager = default_alarm_manager();

        let datetime = DateTime {
            year: 2018,
            month: 10,
            day: 17,
            hour: 9,
            min: 0,
            sec: 0,
            day_of_week: DayOfWeek::Wednesday,
        };

        let next = alarm_manager.next_ring(&datetime);
        assert_eq!(next, Some((DayOfWeek::Thursday, 7, 25)));
    }

    #[test]
    fn test_next_ring_one_time() {
        let mut alarm_manager = default_alarm_manager();
        alarm_manager.alarms[2].is_enable = true;
        alarm_manager.alarms[2].set_hour(17);
        alarm_manager.alarms[2].set_min(30);
        alarm_manager.alarms[2].mode.insert(Mode::ONE_TIME);

        let datetime = DateTime {
            year: 2018,
            month: 12,
            day: 1,
            hour: 17,
            min: 21,
            sec: 0,
            day_of_week: DayOfWeek::Saturday,
        };

        let next = alarm_manager.next_ring(&datetime);
        assert_eq!(next, Some((DayOfWeek::Saturday, 17, 30)));
    }

    #[test]
    fn test_next_ring_never() {
        let mut alarm_manager = default_alarm_manager();
        alarm_manager.alarms[2].is_enable = true;
        alarm_manager.alarms[2].set_hour(17);
        alarm_manager.alarms[2].set_min(30);
        alarm_manager.alarms[2].mode = Mode::empty();

        let datetime = DateTime {
            year: 2018,
            month: 12,
            day: 1,
            hour: 17,
            min: 21,
            sec: 0,
            day_of_week: DayOfWeek::Saturday,
        };

        let next = alarm_manager.next_ring(&datetime);
        assert_eq!(next, Some((DayOfWeek::Monday, 7, 25)));
    }
}
