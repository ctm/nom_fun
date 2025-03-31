// TODO: rename this module.  It never had a good name, but it's even more
//       ridiculous now that most of the duration parsing is now in
//       sports-metrics.

use digital_duration_nom::duration::{Duration, duration_parser};

use nom::{
    IResult, Parser,
    bytes::complete::{tag, take},
    combinator::map,
    multi::{many_till, many0},
    sequence::terminated,
};

fn pace_duration_pair(input: &str) -> IResult<&str, (Duration, Duration)> {
    (
        terminated(duration_parser, tag("(")),
        terminated(duration_parser, tag(")")),
    )
        .parse(input)
}

fn eventual_pace_duration_pair(input: &str) -> IResult<&str, (Duration, Duration)> {
    map(many_till(take(1usize), pace_duration_pair), |pair| pair.1).parse(input)
}

pub fn many_pace_duration_pairs(input: &str) -> IResult<&str, Vec<(Duration, Duration)>> {
    many0(eventual_pace_duration_pair).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pace_duration_pair() {
        assert_eq!(
            (Duration::new_min_sec(8, 9), Duration::new_min_sec(1, 15)),
            pace_duration_pair("8:09(1:15.0)").unwrap().1
        );
    }

    #[test]
    fn test_eventual_pace_duration_pair() {
        assert_eq!(
            (Duration::new_min_sec(8, 9), Duration::new_min_sec(1, 15)),
            eventual_pace_duration_pair("12/24 8:09(1:15.0)").unwrap().1
        );
    }
}
