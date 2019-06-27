// TODO: rename this module.  It never had a good name, but it's even more
//       ridiculous now that most of the duration parsing is now in
//       sports-metrics.

use sports_metrics::duration::{duration_parser, Duration};

use nom::{
    bytes::complete::{tag, take},
    multi::{many0, many_till},
    IResult,
};

fn pace_duration_pair(input: &str) -> IResult<&str, (Duration, Duration)> {
    let (input, pace) = duration_parser(input)?;
    let (input, _) = tag("(")(input)?;
    let (input, duration) = duration_parser(input)?;
    let (input, _) = tag(")")(input)?;
    Ok((input, (pace, duration)))
}

fn eventual_pace_duration_pair(input: &str) -> IResult<&str, (Duration, Duration)> {
    let (input, pair) = many_till(take(1usize), pace_duration_pair)(input)?;
    Ok((
        input,
        (match pair {
            (_, pd_pair) => pd_pair,
        }),
    ))
}

pub fn many_pace_duration_pairs(input: &str) -> IResult<&str, Vec<(Duration, Duration)>> {
    let (input, res) = many0(eventual_pace_duration_pair)(input)?;
    Ok((input, res))
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
