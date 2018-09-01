use heapless::consts::*;
use heapless::LinearMap;
use rtc::datetime::{DateTime, DayOfWeek};

pub struct AlarmManager {
    pub is_enable: bool,
    hour: u8,
    min: u8,
    pub mode: Mode,
}
pub enum Mode {
    OneTime,
    Repeat(LinearMap<DayOfWeek, (), U8>),
}
impl Default for AlarmManager {
    fn default() -> Self {
        AlarmManager {
            is_enable: false,
            hour: 12,
            min: 0,
            mode: Mode::OneTime,
        }
    }
}
impl AlarmManager {
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
        match &self.mode {
            Mode::OneTime => {
                self.is_enable = false;
                true
            }
            Mode::Repeat(days) if days.contains_key(&datetime.day_of_week) => true,
            _ => false,
        }
    }
}
