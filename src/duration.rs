use std::fmt;

pub const SECONDS_IN_MINUTE: u64 = 60;
const MINUTES_IN_HOUR: u64 = 60;
pub const SECONDS_IN_HOUR: u64 = MINUTES_IN_HOUR * SECONDS_IN_MINUTE;

custom_derive! {
    // Can't use NewtypeSum w/o unstable
    #[derive(Debug, PartialEq, NewtypeAdd, NewtypeDiv(u32), NewtypeDeref)]
    pub struct Duration(std::time::Duration);
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let all_secs = self.as_secs();
        let hours = all_secs / SECONDS_IN_HOUR;
        let minutes = all_secs / SECONDS_IN_MINUTE % SECONDS_IN_MINUTE;
        let seconds = all_secs % SECONDS_IN_MINUTE;
        let tenths = self.subsec_millis() / 100 as u32;

        let precision = match f.precision() {
            Some(p) => p,
            None => 0,
        };

        let width = match f.width() {
            Some(w) => w,
            None => 0,
        };

        let mut result = String::new();

        if hours > 0 {
            result.push_str(&format!("{}:{:02}:{:02}", hours, minutes, seconds));
        } else if minutes > 0 {
            result.push_str(&format!("{}:{:02}", minutes, seconds));
        } else {
            result.push_str(&format!("{}", seconds));
        }

        if tenths > 0 || precision > 0 {
            result.push_str(&format!(".{}", tenths));
        }
        write!(f, "{:>width$}", result, width=width)
    }
}

#[test]
fn test_display() {
    assert_eq!(format!("{:7}", Duration::new(35, 0)), "     35");
    assert_eq!(format!("{:7}", Duration::new_min_sec(49, 32)), "  49:32");
    assert_eq!(format!("{:7.1}", Duration::new_min_sec_tenths(9, 12, 3)), " 9:12.3");
}

impl Duration {
    pub fn new(secs: u64, nanos: u32) -> Self {
        Duration(std::time::Duration::new(secs, nanos))
    }

    pub fn new_min_sec(mins: u64, secs: u8) -> Self {
        Self::new_min_sec_tenths(mins, secs, 0)
    }

    pub fn new_min_sec_tenths(mins: u64, secs: u8, tenths: u8) -> Self {
        Self::new(mins * SECONDS_IN_MINUTE + u64::from(secs), u32::from(tenths) * 100_000_000)
    }
}

impl From<f64> for Duration {
    fn from(f64: f64) -> Self {
        Self::new(f64.trunc() as u64, (f64.fract() * 1e9) as u32)
    }
}

impl From<time::Duration> for Duration {
    fn from(duration: time::Duration) -> Self {
        let nanos = duration.num_nanoseconds().unwrap() % 1_000_000_000;

        Self::new(duration.num_seconds() as u64, nanos as u32)
    }
}

impl<'a> std::iter::Sum<&'a Duration> for Duration {
    fn sum<I: Iterator<Item=&'a Duration>>(iter: I) -> Duration {
        Duration(iter.map(|d| d.0).sum())
    }
}
