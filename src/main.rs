#[macro_use] extern crate nom;
#[macro_use] extern crate nom_trace;

use std::time::Duration;
use nom::digit;

//adds a thread local storage object to store the trace
declare_trace!();

pub fn main() {
  named!(parser<&str, Vec<&str>>,
    //wrap a parser with tr!() to add a trace point
    tr!(preceded!(
      tr!(tag!("data: ")),
      tr!(delimited!(
        tag!("("),
        separated_list!(
          tr!(tag!(",")),
          tr!(nom::digit)
        ),
        tr!(tag!(")"))
      ))
    ))
  );

  println!("parsed: {:?}", parser(&"data: (1,2,3)"[..]));

  // prints the last parser trace
  print_trace!();

  // the list of trace events can be cleared
  reset_trace!();
}

#[derive(Debug,PartialEq)]
pub struct Color {
  pub red:   u8,
  pub green: u8,
  pub blue:  u8,
}

#[allow(dead_code)]
fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
  u8::from_str_radix(input, 16)
}

#[allow(dead_code)]
fn is_hex_digit(c: char) -> bool {
  c.is_digit(16)
}

named!(hex_primary<&str, u8>,
  map_res!(take_while_m_n!(2, 2, is_hex_digit), from_hex)
);

named!(hex_color<&str, Color>,
  do_parse!(
           tag!("#")   >>
    red:   hex_primary >>
    green: hex_primary >>
    blue:  hex_primary >>
    (Color { red, green, blue })
  )
);

#[test]
fn parse_color() {
  assert_eq!(hex_color("#2F14DF"), Ok(("", Color {
    red: 47,
    green: 20,
    blue: 223,
  })));
}


//    8:22
//    1:15.0
// 2:25:36
//   20:29.8
//   11:06

// What about
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
// PREFIX_AND_DOUBLE_DIGIT_SECONDS = /#{MINUTE_PREFIX}#{DOUBLE_DIGIT_SECONDS}/
// WITHOUT_DECIMAL = /#{PREFIX_AND_DOUBLE_DIGIT_SECONDS}|#{SINGLE_DIGIT_SECONDS}/
// ALL = /#{WITHOUT_DECIMAL}#{TENTHS}?/

const SECONDS_IN_MINUTE: u64 = 60;
const MINUTES_IN_HOUR: u64 = 60;
const NANOSECONDS_IN_SECOND: u32 = 1_000_000_000;
const TENTHS_IN_A_NANOSECOND: u32 = NANOSECONDS_IN_SECOND / 10;

// So, nom::digit recognizes one *or more* digits.  I don't yet know how
// to recognize exactly one digit.  Ugh!

// Heh! earlier I had is_a!("0123456789") until I found digit.  Now, looking
// through the source I find:
//
//     block_named!(digit, is_a!("0123456789"));
//
// Since that's the case, I don't feel bad about defining my own single digit,
// if I don't find something better.
//
// Oh, hey, looks like there may be an i64 block that I can use that will
// scoop up the digits and then do the parsing for me.  I should look to
// see if there is a u64 or anything similar.  This will require some poking
// around and likely some experimentation, too.
//
// HMMMM... I may have prematurely jumped to conclusions.  I don't know
// that the above block_named! stuff is actually defining digit.  It could
// be something else.
//
// Looks like this is it:
//
//     complete_named!(digit, is_a!("0123456789"));

named!(hour_prefix<&str, Duration>,
  do_parse!(
    digits: digit >>  
    tag!(":") >>
    (Duration::new(digits.parse::<u64>().unwrap() *
                   MINUTES_IN_HOUR * SECONDS_IN_MINUTE, 0))
  )
);

#[test]
fn test_hour_prefix() {
    assert_eq!(Duration::new(3600, 0), hour_prefix("1:").unwrap().1);
    assert_eq!(Duration::new(36000, 0), hour_prefix("10:").unwrap().1);
}

named!(zero_through_five<&str, u8>,
  do_parse!(
    digit: one_of!("012345") >>
    (digit as u8 - b'0')
  )
);

named!(single_digit<&str, u8>,
  do_parse!(
    digit: one_of!("0123456789") >>
    (digit as u8 - b'0')
  )
);

named!(double_digit_minute_prefix<&str, Duration>,
  do_parse!(
    tens: zero_through_five >>
    ones: single_digit >>
    tag!(":") >>
    (Duration::new(((tens as u64 * 10) + ones as u64) * SECONDS_IN_MINUTE, 0))
  )
);

named!(single_digit_minute_prefix<&str, Duration>,
  do_parse!(
    ones: single_digit >>
    tag!(":") >>
    (Duration::new(ones as u64 * SECONDS_IN_MINUTE, 0))
  )
);

named!(double_digit_seconds<&str, Duration>,
  do_parse!(
    tens: zero_through_five >>
    ones: single_digit >>
    (Duration::new((tens as u64 * 10) + ones as u64, 0))
  )
);

named!(single_digit_seconds<&str, Duration>,
  do_parse!(
    ones: single_digit >>
    (Duration::new(ones as u64, 0))
  )
);

named!(tenths<&str, Duration>,
  do_parse!(
    tag!(".") >>
    tenth: single_digit >>
    (Duration::new(0, tenth as u32 * TENTHS_IN_A_NANOSECOND))
  )
);

#[test]
fn test_tenths() {
    assert_eq!(Duration::new(0, 900_000_000), tenths(".9").unwrap().1);
    assert_eq!(Duration::new(1, 0), tenths(".9").unwrap().1 +
                                    tenths(".1").unwrap().1);
}

named!(hour_and_minute_prefix<&str, Duration>,
  do_parse!(
    hours: opt!(hour_prefix) >>
    minutes: double_digit_minute_prefix >>
    (match hours {
      None => minutes,
      Some(hours) => hours + minutes,
    })
  )
);

named!(minute_prefix<&str, Duration>,
  alt!(hour_and_minute_prefix | single_digit_minute_prefix)
);

named!(prefix_and_double_digit_seconds<&str, Duration>,
  do_parse!(
    minutes: minute_prefix >>
    seconds: double_digit_seconds >>
    (minutes + seconds)
  )
);

named!(without_decimal<&str, Duration>,
  alt!(prefix_and_double_digit_seconds | single_digit_seconds)
);

// TODO: all is a horrible name!
named!(all<&str, Duration>,
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
fn test_all() {
    assert_eq!(Duration::new(8 * SECONDS_IN_MINUTE + 22, 0),
               all("8:22").unwrap().1);
}
    
