// TODO: rename this module.  It never had a good name, but it's even more
//       ridiculous now that most of the duration parsing is now in
//       sports-metrics.

use nom::types::CompleteStr;

use sports_metrics::duration::{duration_parser, Duration};

named!(pace_duration_pair<CompleteStr, (Duration, Duration)>,
  do_parse!(
    pace: duration_parser >>
    char!('(') >>
    duration: duration_parser >>
    char!(')') >>
    ((pace, duration))
  )
);

named!(eventual_pace_duration_pair<CompleteStr, (Duration, Duration)>,
  do_parse!(
    pair: many_till!(take!(1), pace_duration_pair) >>
    (match pair {
      (_, pd_pair) => pd_pair
    })
  )
);

named!(pub many_pace_duration_pairs<CompleteStr, Vec<(Duration, Duration)>>,
  many0!(eventual_pace_duration_pair)
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pace_duration_pair() {
        assert_eq!(
            (Duration::new_min_sec(8, 9), Duration::new_min_sec(1, 15)),
            pace_duration_pair(CompleteStr("8:09(1:15.0)")).unwrap().1
        );
    }

    #[test]
    fn test_eventual_pace_duration_pair() {
        assert_eq!(
            (Duration::new_min_sec(8, 9), Duration::new_min_sec(1, 15)),
            eventual_pace_duration_pair(CompleteStr("12/24 8:09(1:15.0)"))
                .unwrap()
                .1
        );
    }

}
