use nom::digit;
use nom::types::CompleteStr;

use crate::duration::Duration;

//    8:22
//    1:15.0
// 2:25:36
//   20:29.8
//   11:06
//       0
//       1
//      05
//      10

// Here's some ruby regexp code that shows what I'm going for

// HOUR_PREFIX = /([0-9]+):/
// DOUBLE_DIGIT_MINUTE_PREFIX = /([0-5][0-9]):/
// SINGLE_DIGIT_MINUTE_PREFIX = /([0-9]):/
// DOUBLE_DIGIT_SECONDS = /([0-5][0-9])/
// SINGLE_DIGIT_SECONDS = /([0-9])/
// TENTHS = /\.([0-9])/
// HOUR_AND_MINUTE_PREFIX = /#{HOUR_PREFIX}?#{DOUBLE_DIGIT_MINUTE_PREFIX}/
// MINUTE_PREFIX = /#{HOUR_AND_MINUTE_PREFIX}|#{SINGLE_DIGIT_MINUTE_PREFIX}/
// PREFIX_AND_DOUBLE_DIGIT_SECONDS = /#{MINUTE_PREFIX}?#{DOUBLE_DIGIT_SECONDS}/
// WITHOUT_DECIMAL = /#{PREFIX_AND_DOUBLE_DIGIT_SECONDS}|#{SINGLE_DIGIT_SECONDS}/
// ALL = /#{WITHOUT_DECIMAL}#{TENTHS}?/

const SECONDS_IN_MINUTE: u64 = crate::duration::SECONDS_IN_MINUTE;
const SECONDS_IN_HOUR: u64 = crate::duration::SECONDS_IN_HOUR;
const NANOSECONDS_IN_SECOND: u32 = 1_000_000_000;
const TENTHS_IN_NANOSECOND: u32 = NANOSECONDS_IN_SECOND / 10;

named!(hour_prefix<CompleteStr, Duration>,
  do_parse!(
    digits: digit >>
    tag!(":") >>
    (Duration::new(digits.parse::<u64>().unwrap() * SECONDS_IN_HOUR, 0))
  )
);

#[test]
fn test_hour_prefix() {
    assert_eq!(
        Duration::new(3600, 0),
        hour_prefix(CompleteStr("1:")).unwrap().1
    );
    assert_eq!(
        Duration::new(36000, 0),
        hour_prefix(CompleteStr("10:")).unwrap().1
    );
}

named!(zero_through_five<CompleteStr, u8>,
  do_parse!(
    digit: one_of!("012345") >>
    (digit as u8 - b'0')
  )
);

named!(single_digit<CompleteStr, u8>,
  do_parse!(
    digit: one_of!("0123456789") >>
    (digit as u8 - b'0')
  )
);

named!(double_digit_minute_prefix<CompleteStr, Duration>,
  do_parse!(
    tens: zero_through_five >>
    ones: single_digit >>
    tag!(":") >>
    (Duration::new(((u64::from(tens) * 10) + u64::from(ones)) * SECONDS_IN_MINUTE, 0))
  )
);

#[test]
fn test_double_digit_minute_prefix() {
    assert_eq!(
        Duration::new(11 * SECONDS_IN_MINUTE, 0),
        double_digit_minute_prefix(CompleteStr("11:06")).unwrap().1
    );
}

named!(single_digit_minute_prefix<CompleteStr, Duration>,
  do_parse!(
    ones: single_digit >>
    tag!(":") >>
    (Duration::new(u64::from(ones) * SECONDS_IN_MINUTE, 0))
  )
);

named!(double_digit_seconds<CompleteStr, Duration>,
  do_parse!(
    tens: zero_through_five >>
    ones: single_digit >>
    (Duration::new((u64::from(tens) * 10) + u64::from(ones), 0))
  )
);

named!(single_digit_seconds<CompleteStr, Duration>,
  do_parse!(
    ones: single_digit >>
    (Duration::new(u64::from(ones), 0))
  )
);

named!(tenths<CompleteStr, Duration>,
  do_parse!(
    tag!(".") >>
    tenth: single_digit >>
    (Duration::new(0, u32::from(tenth) * TENTHS_IN_NANOSECOND))
  )
);

#[test]
fn test_tenths() {
    assert_eq!(
        Duration::new(0, 900_000_000),
        tenths(CompleteStr(".9")).unwrap().1
    );
    assert_eq!(
        Duration::new(1, 0),
        tenths(CompleteStr(".9")).unwrap().1 + tenths(CompleteStr(".1")).unwrap().1
    );
}

named!(hour_and_minute_prefix<CompleteStr, Duration>,
  alt!(
    do_parse!(
      hours: hour_prefix >>
      minutes: double_digit_minute_prefix >>
      (hours + minutes)
    ) |
    double_digit_minute_prefix
  )
);

#[test]
fn test_hour_and_minute_prefix() {
    assert_eq!(
        Duration::new(11 * SECONDS_IN_MINUTE, 0),
        hour_and_minute_prefix(CompleteStr("11:06")).unwrap().1
    );
}

named!(minute_prefix<CompleteStr, Duration>,
  alt!(hour_and_minute_prefix | single_digit_minute_prefix)
);

#[test]
fn test_minute_prefix() {
    assert_eq!(
        Duration::new(11 * SECONDS_IN_MINUTE, 0),
        minute_prefix(CompleteStr("11:06")).unwrap().1
    );
}

named!(prefix_and_double_digit_seconds<CompleteStr, Duration>,
  do_parse!(
    minutes: opt!(minute_prefix) >>
    seconds: double_digit_seconds >>
    (match minutes {
      None => seconds,
      Some(minutes) => minutes + seconds
    })
  )
);

#[test]
fn test_prefix_and_double_digit_seconds() {
    assert_eq!(
        Duration::new(11 * SECONDS_IN_MINUTE + 6, 0),
        prefix_and_double_digit_seconds(CompleteStr("11:06"))
            .unwrap()
            .1
    );
}

named!(without_decimal<CompleteStr, Duration>,
  alt!(prefix_and_double_digit_seconds | single_digit_seconds)
);

// There's probably a better name than duration, but then again, calling
// this module interval_parse is sub-optimal, too.
named!(pub duration<CompleteStr, Duration>,
  do_parse!(
    seconds: without_decimal >>
    tenths: opt!(tenths) >>
    (match tenths {
      None => seconds,
      Some(tenths) => seconds + tenths,
    })
  )
);

#[test]
fn test_duration() {
    assert_eq!(
        Duration::new(8 * SECONDS_IN_MINUTE + 22, 0),
        duration(CompleteStr("8:22")).unwrap().1
    );

    assert_eq!(
        Duration::new(1 * SECONDS_IN_MINUTE + 15, 3 * TENTHS_IN_NANOSECOND),
        duration(CompleteStr("1:15.3")).unwrap().1
    );

    assert_eq!(
        Duration::new(2 * SECONDS_IN_HOUR + 25 * SECONDS_IN_MINUTE + 36, 0),
        duration(CompleteStr("2:25:36")).unwrap().1
    );

    assert_eq!(
        Duration::new(
            2 * SECONDS_IN_HOUR + 25 * SECONDS_IN_MINUTE + 36,
            7 * TENTHS_IN_NANOSECOND
        ),
        duration(CompleteStr("2:25:36.7")).unwrap().1
    );

    assert_eq!(
        Duration::new(20 * SECONDS_IN_MINUTE + 29, 8 * TENTHS_IN_NANOSECOND),
        duration(CompleteStr("20:29.8")).unwrap().1
    );

    assert_eq!(
        Duration::new(11 * SECONDS_IN_MINUTE + 6, 0),
        duration(CompleteStr("11:06")).unwrap().1
    );

    assert_eq!(Duration::new(0, 0), duration(CompleteStr("0")).unwrap().1);

    assert_eq!(Duration::new(1, 0), duration(CompleteStr("1")).unwrap().1);

    assert_eq!(Duration::new(5, 0), duration(CompleteStr("05")).unwrap().1);

    assert_eq!(Duration::new(10, 0), duration(CompleteStr("10")).unwrap().1);

    assert_eq!(Duration::new(8 * SECONDS_IN_MINUTE + 1, 6 * TENTHS_IN_NANOSECOND),
               duration(CompleteStr("8:01.6")).unwrap().1);
}

named!(pace_duration_pair<CompleteStr, (Duration, Duration)>,
  do_parse!(
    pace: duration >>
    char!('(') >>
    duration: duration >>
    char!(')') >>
    ((pace, duration))
  )
);

#[test]
fn test_pace_duration_pair() {
    assert_eq!(
        (Duration::new(8 * SECONDS_IN_MINUTE +  9, 0),
         Duration::new(1 * SECONDS_IN_MINUTE + 15, 0)),
        pace_duration_pair(CompleteStr("8:09(1:15.0)")).unwrap().1
    );
}

named!(eventual_pace_duration_pair<CompleteStr, (Duration, Duration)>,
  do_parse!(
    pair: many_till!(take!(1), pace_duration_pair) >>
    (match pair {
      (_, pd_pair) => pd_pair
    })
  )
);

#[test]
fn test_eventual_pace_duration_pair() {
    assert_eq!(
        (Duration::new(8 * SECONDS_IN_MINUTE +  9, 0),
         Duration::new(1 * SECONDS_IN_MINUTE + 15, 0)),
        eventual_pace_duration_pair(CompleteStr("12/24 8:09(1:15.0)")).unwrap().1
    );
}

named!(pub many_pace_duration_pairs<CompleteStr, Vec<(Duration, Duration)>>,
  many0!(eventual_pace_duration_pair)
);
