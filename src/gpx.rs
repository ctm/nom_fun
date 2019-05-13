use std::str::FromStr;

use chrono::{DateTime, Utc};
use ordered_float::NotNan;
use roxmltree::Document;
use roxmltree::Node;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

// FWIW, we use time::Duration here rather than
// sports_metrics::duration::Duration because we need to be able to
// add durations to (and subtract durations from)
// chrono::datetime::DateTimes and sports_metrics currently
// deliberately avoids time and chrono dependencies.
use time::Duration;

// TODO: figure out the interval duration looking for abrupt changes in
//       speed.  Consider having a constant like 5 for the common factor
//       that all intervals will have.

const METERS_PER_MILE: f64 = 1609.344;
const SECONDS_PER_MINUTE: f64 = 60.0;

#[derive(Debug)]
pub struct Gpx {
    trkpts: Vec<Trkpt>,
}

#[derive(Debug)]
pub struct Trkpt {
    pub time: DateTime<Utc>,
    meters_per_second: Option<f64>,
    meters: Option<f64>,
    heart_rate: Option<u8>,
    cadence: Option<u8>,
    elevation_meters: Option<f64>,
    vertical_mps: Option<f64>,
}

#[derive(Debug, PartialOrd, Clone)]
struct Interval {
    rank: NotNan<f64>, // meters_per_second, adjusted by elevation changes
    minutes_per_mile: f64,
    start: DateTime<Utc>,
    stop: DateTime<Utc>,
    gain: f64,
    loss: f64,
}

impl Ord for Interval {
    fn cmp(&self, other: &Interval) -> Ordering {
        match self.rank.cmp(&other.rank) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => self.start.cmp(&other.start),
        }
    }
}

impl PartialEq for Interval {
    fn eq(&self, other: &Interval) -> bool {
        self.rank == other.rank && self.start == other.start
    }
}

impl Eq for Interval {}

impl Trkpt {
    fn f64_from_node(node: &Node) -> Option<f64> {
        Some(f64::from_str(node.text().unwrap()).unwrap())
    }

    fn u8_from_node(node: &Node) -> Option<u8> {
        Some(u8::from_str(node.text().unwrap()).unwrap())
    }

    fn from_node(node: &Node) -> Self {
        let mut time = None;
        let mut meters_per_second = None;
        let mut meters = None;
        let mut heart_rate = None;
        let mut cadence = None;
        let mut elevation_meters = None;
        let mut vertical_mps = None;

        for elem in node.descendants() {
            match elem.tag_name().name() {
                "time" => time = Some(DateTime::<Utc>::from_str(elem.text().unwrap()).unwrap()),
                "speed" => meters_per_second = Self::f64_from_node(&elem),
                "distance" => meters = Self::f64_from_node(&elem),
                "hr" => heart_rate = Self::u8_from_node(&elem),
                "cadence" => cadence = Self::u8_from_node(&elem),
                "altitude" => elevation_meters = Self::f64_from_node(&elem),
                "verticalSpeed" => vertical_mps = Self::f64_from_node(&elem),
                _ => (),
            }
        }

        match time {
            None => panic!("Trackpoint without time"),
            Some(time) => Trkpt {
                time,
                meters_per_second,
                meters,
                heart_rate,
                cadence,
                elevation_meters,
                vertical_mps,
            },
        }
    }
}

impl Gpx {
    pub fn from_string(string: &str) -> Self {
        let trkpts = Self::trkpts(&Document::parse(string).unwrap());
        Gpx { trkpts }
    }

    pub fn trkpt_iterator<'a>(doc: &'a Document) -> impl Iterator<Item = Trkpt> + 'a {
        doc.descendants()
            .next()
            .unwrap()
            .descendants()
            .find(|n| n.has_tag_name("trkseg"))
            .unwrap()
            .descendants()
            .filter(|n| n.has_tag_name("trkpt"))
            .map(|trkpt| Trkpt::from_node(&trkpt))
    }

    fn trkpts(doc: &Document) -> Vec<Trkpt> {
        Self::trkpt_iterator(doc).collect()
    }

    fn f64_duration(duration: &Duration) -> f64 {
        duration.num_nanoseconds().unwrap() as f64 * 1e-9
    }

    // I'm not convinced that fudging by elevation is the way to go.
    // It makes more sense to compare speed on either side of a
    // potential interval, although that could give us false positives
    // with short descents.
    //
    // OTOH, I have about nine months of Monday Intervals.  I used to
    // do them at fixed points, so the amount of rest between the
    // interval would vary from week to week, and my interval duration
    // was initially just a minute.  My guess is that if I add a
    // couple of parameters to the detection routine that I can get by
    // just with the gain fudge.
    fn gain_fudge(gain_per_second: f64) -> f64 {
        // The min here is mostly due to the possibility of a false
        // value due to the way elevation works (at least with my
        // Ambit 3).  Specifically, I added it after trying to suss
        // the intervals out of my 2018_07_02 file and having it fail
        // due to an incorrect gain.
        1.0 + (gain_per_second / 0.062_511).min(1.0) * 0.15
    }

    // TODO: get rid of loss_fudge once the parameterized version of
    // the interval finder is working with all my sample data.
    fn loss_fudge(_loss_per_second: f64) -> f64 {
        1.0
    }

    fn mpm_from_mps(meters_per_second: f64) -> f64 {
        METERS_PER_MILE / SECONDS_PER_MINUTE / meters_per_second
    }

    fn potential_intervals(&self, duration: u8) -> BinaryHeap<Interval> {
        let mut intervals = BinaryHeap::<Interval>::new();
        let interval_duration = Duration::seconds(i64::from(duration));

        for window in self.trkpts.windows(duration as usize + 1) {
            let mut window = window.iter();
            if let Some(trkpt) = window.next() {
                let start = trkpt.time;
                let mut meters = 0.0;
                let mut duration = Duration::seconds(0);
                let mut last_time = start;
                let mut gain = 0.0;
                let mut loss = 0.0;

                while duration < interval_duration {
                    if let Some(trkpt) = window.next() {
                        if let Some(meters_per_second) = trkpt.meters_per_second {
                            let vertical_mps = match trkpt.vertical_mps {
                                Some(vertical_mps) => vertical_mps,
                                None => 0.0,
                            };
                            let time = trkpt.time;
                            let delta = time - last_time;
                            let f64_delta = Self::f64_duration(&delta);
                            meters += f64_delta * meters_per_second;
                            let change = f64_delta * vertical_mps;
                            if change.is_sign_negative() {
                                loss -= change;
                            } else {
                                gain += change;
                            }
                            // duration += delta; AddAssign not implemented!
                            duration = duration + delta;
                            last_time = time;
                        }
                    } else {
                        break;
                    }
                }

                if duration >= interval_duration {
                    let stop = start + duration;
                    let f64_duration = Self::f64_duration(&duration);
                    let meters_per_second = meters / f64_duration;
                    let gain_per_second = gain / f64_duration;
                    let loss_per_second = loss / f64_duration;
                    let rank = NotNan::new(
                        meters_per_second
                            * Self::gain_fudge(gain_per_second)
                            * Self::loss_fudge(loss_per_second),
                    )
                    .unwrap();
                    let minutes_per_mile = Self::mpm_from_mps(meters_per_second);
                    intervals.push(Interval {
                        rank,
                        minutes_per_mile,
                        start,
                        stop,
                        gain,
                        loss,
                    })
                }
            }
        }

        intervals
    }

    // TODO: I didn't see a trivial way to see if two Ranges intersected.
    //       That's a bit surprising, but perhaps I'm just not good enough
    //       at searching out Rust methods.  Perhaps because it's so easy
    //       to do ourselves.  Still, eventually I should ask someone how
    //       to figure out if I'm overrlooking a method, especially one
    //       that seems likely to exist.
    fn contains(intervals: &[Interval], interval: &Interval) -> bool {
        intervals
            .iter()
            .any(|i| interval.start < i.stop && interval.stop > i.start)
    }

    fn dump(&self, intervals: &[Interval]) {
        // TODO: document total_pace_durations
        let mut total_pace_durations = sports_metrics::duration::Duration::new(0, 0);
        let mut total_elapsed = sports_metrics::duration::Duration::new(0, 0);

        for interval in intervals {
            let seconds_per_mile = interval.minutes_per_mile * SECONDS_PER_MINUTE;
            let pace = sports_metrics::duration::Duration::from(seconds_per_mile);
            let elapsed = interval.stop - interval.start;
            let elapsed = sports_metrics::duration::Duration::from(Self::f64_duration(&elapsed));
            total_pace_durations += pace * elapsed;
            total_elapsed += elapsed;
            let rank = interval.rank;
            let gain = interval.gain;
            let loss = interval.loss;
            println!(
                "{:.6} {:7} {:7.1} {:.5} {:.5}",
                rank, elapsed, pace, gain, loss
            );
        }

        let average = total_pace_durations / total_elapsed.as_secs() as u32;
        println!("Average: {}", average);
    }

    fn trim(intervals: &mut Vec<Interval>, count: u8) {
        let mut len = intervals.len() as u8;

        while len > count {
            if intervals.first().unwrap().rank < intervals.last().unwrap().rank {
                intervals.remove(0);
            } else {
                intervals.pop();
            }
            len -= 1;
        }
    }

    fn restrict_to_actual_intervals(intervals: &mut Vec<Interval>, span: f32, count: u8) {
        let span_with_slop = Duration::seconds((span * 1.50) as i64);
        let mut results = Vec::with_capacity(count as usize);
        let best = intervals[0].clone();

        intervals.sort_by_key(|i| i.start);

        let mut start_idx = intervals
            .iter()
            .position(|interval| *interval == best)
            .unwrap();
        let mut stop_idx = start_idx + 1;

        let mut expected_start = best.start - span_with_slop;
        let min_rank = best.rank * 0.70; // TODO: document!
        while start_idx > 0
            && intervals[start_idx - 1].start >= expected_start
            && intervals[start_idx - 1].rank >= min_rank
        {
            start_idx -= 1;
            expected_start = intervals[start_idx].start - span_with_slop;
        }

        let max_stop_idx = intervals.len();
        expected_start = best.start + span_with_slop;
        while stop_idx < max_stop_idx
            && intervals[stop_idx].start <= expected_start
            && intervals[stop_idx].rank >= min_rank
        {
            stop_idx += 1;
            if stop_idx < max_stop_idx {
                expected_start = intervals[stop_idx].start + span_with_slop;
            }
        }

        results.extend_from_slice(&intervals[start_idx..stop_idx as usize]);
        // Consider adjusting start_idx and stop_idx before extend_from_slice
        // since adding the intervals to results and then trimming results is
        // less efficient.  It certainly doesn't matter here, but still...
        Self::trim(&mut results, count);
        *intervals = results;
    }

    pub fn analyze(&self, duration: u8, rest: u8, count: u8) {
        let mut heap = self.potential_intervals(duration);
        let mut intervals = Vec::new();

        while let Some(interval) = heap.pop() {
            if !Self::contains(&intervals, &interval) {
                intervals.push(interval);
            }
        }

        Self::restrict_to_actual_intervals(
            &mut intervals,
            f32::from(duration) + f32::from(rest),
            count,
        );

        self.dump(&intervals);

        if intervals.len() != usize::from(count) {
            panic!(
                "Was told to find {} intervals, but found {}",
                count,
                intervals.len()
            );
        }
    }
}
