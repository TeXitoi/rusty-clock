static DAYS_IN_MONTH: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
static DAYS_IN_MONTH_LEAP_YEAR: [u32; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
fn days_in_month(is_leap_year: bool) -> &'static [u32; 12] {
    if is_leap_year {
        &DAYS_IN_MONTH_LEAP_YEAR
    } else {
        &DAYS_IN_MONTH
    }
}
fn is_leap(year: u16) -> bool {
    if year % 4 != 0 {
        false
    } else if year % 100 != 0 {
        true
    } else {
        year % 400 == 0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}
impl DayOfWeek {
    pub fn from_days_since_epoch(days: u32) -> DayOfWeek {
        use self::DayOfWeek::*;
        match days % 7 {
            4 => Monday,
            5 => Tuesday,
            6 => Wednesday,
            0 => Thursday,
            1 => Friday,
            2 => Saturday,
            3 => Sunday,
            _ => unreachable!(),
        }
    }
    pub fn next(self) -> Self {
        use self::DayOfWeek::*;
        match self {
            Monday => Tuesday,
            Tuesday => Wednesday,
            Wednesday => Thursday,
            Thursday => Friday,
            Friday => Saturday,
            Saturday => Sunday,
            Sunday => Monday,
        }
    }
}
impl ::core::fmt::Display for DayOfWeek {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> Result<(), ::core::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub min: u8,
    pub sec: u8,
    pub day_of_week: DayOfWeek,
}
impl DateTime {
    pub fn new(epoch: u32) -> DateTime {
        let mut days = epoch / 86400;
        let time = epoch % 86400;
        let day_of_week = DayOfWeek::from_days_since_epoch(days);
        let mut year = 1970;
        let mut is_leap_year;

        loop {
            is_leap_year = is_leap(year);
            if is_leap_year && days >= 366 {
                year += 1;
                days -= 366
            } else if !is_leap_year && days >= 365 {
                year += 1;
                days -= 365;
            } else {
                break;
            }
        }
        let mut month = 1;
        for &nb in days_in_month(is_leap_year).iter() {
            if days < nb {
                break;
            }
            days -= nb;
            month += 1;
        }
        DateTime {
            year,
            month,
            day: (days + 1) as u8,
            hour: (time / 60 / 60) as u8,
            min: (time / 60 % 60) as u8,
            sec: (time % 60) as u8,
            day_of_week,
        }
    }
    pub fn to_epoch(&self) -> Option<u32> {
        if self.year < 1970 || self.month == 0 || self.month > 12 || self.day == 0 {
            return None;
        }
        let mut days = u32::from(self.day) - 1;
        for y in 1970..self.year {
            if is_leap(y) {
                days = days.checked_add(366)?;
            } else {
                days = days.checked_add(365)?;
            }
        }

        for &m in days_in_month(is_leap(self.year))
            .iter()
            .take(self.month as usize - 1)
        {
            days = days.checked_add(m)?;
        }

        let time = u32::from(self.hour) * 3600 + u32::from(self.min) * 60 + u32::from(self.sec);
        let epoch = days.checked_mul(86400)?.checked_add(time)?;
        Some(epoch)
    }
}
impl ::core::fmt::Display for DateTime {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> Result<(), ::core::fmt::Error> {
        write!(
            f,
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02} ({})",
            self.year, self.month, self.day, self.hour, self.min, self.sec, self.day_of_week,
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const EPOCH: DateTime = DateTime {
        year: 1970,
        month: 1,
        day: 1,
        hour: 0,
        min: 0,
        sec: 0,
        day_of_week: DayOfWeek::Thursday,
    };
    const END_OF_TIME: DateTime = DateTime {
        year: 2106,
        month: 2,
        day: 7,
        hour: 6,
        min: 28,
        sec: 15,
        day_of_week: DayOfWeek::Sunday,
    };

    #[test]
    fn test_epoch() {
        assert_eq!(DateTime::new(0), EPOCH);
        assert_eq!(DateTime::new(core::u32::MAX), END_OF_TIME);
        assert_eq!(
            DateTime::new(1540052501),
            DateTime {
                year: 2018,
                month: 10,
                day: 20,
                hour: 16,
                min: 21,
                sec: 41,
                day_of_week: DayOfWeek::Saturday,
            }
        );
        assert_eq!(EPOCH.to_epoch(), Some(0));
        assert_eq!(END_OF_TIME.to_epoch(), Some(core::u32::MAX));
        assert_eq!(
            DateTime {
                sec: 16,
                ..END_OF_TIME
            }
            .to_epoch(),
            None
        );
        assert_eq!(
            DateTime {
                year: 1969,
                month: 12,
                day: 31,
                hour: 23,
                min: 59,
                sec: 59,
                day_of_week: DayOfWeek::Wednesday,
            }
            .to_epoch(),
            None
        );
    }
}
