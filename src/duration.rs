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
            None => 0
        };

        if hours > 0 {
            write!(f, "{}:{:02}:{:02}", hours, minutes, seconds)?;
        } else if minutes > 0 {
            write!(f, "{}:{:02}", minutes, seconds)?;
        } else {
            write!(f, "{}", seconds)?;
        }

        if tenths > 0 || precision > 0 {
            write!(f, ".{}", tenths)?;
        }
        write!(f, "")
    }
}

impl Duration {
    pub fn new(secs: u64, nanos: u32) -> Self {
        Duration(std::time::Duration::new(secs, nanos))
    }
}

impl<'a> std::iter::Sum<&'a Duration> for Duration {
    fn sum<I: Iterator<Item=&'a Duration>>(iter: I) -> Duration {
        Duration(iter.map(|d| d.0).sum())
    }
}
