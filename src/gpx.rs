use chrono::{DateTime, TimeDelta, Utc};
use digital_duration_nom::duration::Duration;
use geo::{LineString, prelude::*};
use ordered_float::NotNan;
use roxmltree::Document;
use roxmltree::Node;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::str::FromStr;

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
    #[allow(dead_code)]
    meters: Option<f64>,
    #[allow(dead_code)]
    heart_rate: Option<u8>,
    #[allow(dead_code)]
    cadence: Option<u8>,
    #[allow(dead_code)]
    elevation_meters: Option<f64>,
    vertical_mps: Option<f64>,
    lat: f64,
    lon: f64,
}

#[derive(Debug, Clone)]
struct Interval {
    rank: NotNan<f64>, // meters_per_second, adjusted by elevation changes
    minutes_per_mile: f64,
    start: DateTime<Utc>,
    stop: DateTime<Utc>,
    gain: f64,
    loss: f64,
}

impl Ord for Interval {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.rank.cmp(&other.rank) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => self.start.cmp(&other.start),
        }
    }
}

impl PartialOrd for Interval {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Interval {
    fn eq(&self, other: &Self) -> bool {
        self.rank == other.rank && self.start == other.start
    }
}

impl Eq for Interval {}

impl Trkpt {
    fn f64_from_node(node: &Node) -> f64 {
        f64::from_str(node.text().unwrap()).unwrap()
    }

    fn u8_from_node(node: &Node) -> u8 {
        u8::from_str(node.text().unwrap()).unwrap()
    }

    fn from_node(node: &Node) -> Self {
        let mut time = None;
        let mut meters_per_second = None;
        let mut meters = None;
        let mut heart_rate = None;
        let mut cadence = None;
        let mut elevation_meters = None;
        let mut vertical_mps = None;
        let lat = node.attribute("lat").unwrap().parse().unwrap();
        let lon = node.attribute("lon").unwrap().parse().unwrap();

        for elem in node.descendants() {
            if elem.text().is_none() {
                // TODO: change this to an if let and
                continue; // then use the value
            }
            match elem.tag_name().name() {
                "time" => time = Some(DateTime::<Utc>::from_str(elem.text().unwrap()).unwrap()),
                "speed" => meters_per_second = Some(Self::f64_from_node(&elem)),
                "distance" => meters = Some(Self::f64_from_node(&elem)),
                "hr" | "heartrate" => heart_rate = Some(Self::u8_from_node(&elem)),
                "cadence" => cadence = Some(Self::u8_from_node(&elem)),
                "altitude" | "ele" => elevation_meters = Some(Self::f64_from_node(&elem)),
                "verticalSpeed" => vertical_mps = Some(Self::f64_from_node(&elem)),
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
                lat,
                lon,
            },
        }
    }
}

impl Gpx {
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

    fn f64_duration(duration: &TimeDelta) -> f64 {
        duration.num_nanoseconds().unwrap() as f64 * 1e-9
    }

    fn mpm_from_mps(meters_per_second: f64) -> f64 {
        METERS_PER_MILE / SECONDS_PER_MINUTE / meters_per_second
    }

    fn potential_intervals(&self, duration: u8) -> BinaryHeap<Interval> {
        let mut intervals = BinaryHeap::<Interval>::new();
        let interval_duration = TimeDelta::try_seconds(i64::from(duration)).unwrap();

        // The * 2 is a fudge factor.  With my Ambit 3, we get a
        // little more than one sample per second now that the GPX
        // files are coming from the .sml files running through
        // convert-moves.
        for window in self.trkpts.windows(duration as usize * 2) {
            let mut window = window.iter();
            if let Some(trkpt) = window.next() {
                let start = trkpt.time;
                let mut meters = 0.0;
                let mut duration = TimeDelta::try_seconds(0).unwrap();
                let mut last_time = start;
                let mut gain = 0.0;
                let mut loss = 0.0;

                while duration < interval_duration {
                    if let Some(trkpt) = window.next() {
                        if let Some(meters_per_second) = trkpt.meters_per_second {
                            let vertical_mps = trkpt.vertical_mps.unwrap_or(0.0);
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
                            duration += delta;
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
                    let rank = NotNan::new(meters_per_second).unwrap();
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

    fn precludes(intervals: &[Interval], interval: &Interval, rest: u8) -> bool {
        intervals.iter().any(|i| {
            interval.start < i.stop + std::time::Duration::from_secs((rest / 2).into())
                && interval.stop > i.start
        })
    }

    fn dump(&self, intervals: &[Interval], tod: bool) {
        // TODO: document total_pace_durations
        let mut total_pace_durations = Duration::new(0, 0);
        let mut total_elapsed = Duration::new(0, 0);

        for interval in intervals {
            let seconds_per_mile = interval.minutes_per_mile * SECONDS_PER_MINUTE;
            let pace = Duration::from(seconds_per_mile);
            let elapsed = interval.stop - interval.start;
            let elapsed = Duration::from(Self::f64_duration(&elapsed));
            total_pace_durations += pace * elapsed;
            total_elapsed += elapsed;
            let rank = interval.rank;
            let gain = interval.gain;
            let loss = interval.loss;
            print!("{rank:.6} {elapsed:7} {pace:7.1} {gain:.5} {loss:.5} ");
            if tod {
                println!(
                    "{} {}",
                    interval.start.with_timezone(crate::tz()),
                    interval.stop.with_timezone(crate::tz())
                );
            } else {
                println!(
                    "{:9.1} {:9.1}",
                    self.elapsed(interval.start),
                    self.elapsed(interval.stop)
                );
            }
        }

        let average = total_pace_durations / total_elapsed.as_secs() as u32;
        println!("Average: {}", average);
    }

    fn elapsed(&self, when: DateTime<Utc>) -> Duration {
        let elapsed = when - self.trkpts[0].time;
        Duration::new(
            elapsed.num_seconds().try_into().unwrap(),
            elapsed.subsec_nanos().try_into().unwrap(),
        )
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
        let span_with_slop = TimeDelta::try_seconds((span * 1.50) as i64).unwrap();
        let mut results = Vec::with_capacity(count as usize);
        let best = intervals[0].clone();

        intervals.sort_by_key(|i| i.start);

        let mut start_idx = intervals
            .iter()
            .position(|interval| *interval == best)
            .unwrap();
        let mut stop_idx = start_idx + 1;

        let mut expected_start = best.start - span_with_slop;
        let min_rank = NotNan::new(best.rank * 0.70).unwrap(); // TODO: document!
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

        results.extend_from_slice(&intervals[start_idx..stop_idx]);
        // Consider adjusting start_idx and stop_idx before extend_from_slice
        // since adding the intervals to results and then trimming results is
        // less efficient.  It certainly doesn't matter here, but still...
        Self::trim(&mut results, count);
        *intervals = results;
    }

    fn intervals(&self, duration: u8, rest: u8, count: u8) -> Vec<Interval> {
        let mut heap = self.potential_intervals(duration);
        let mut intervals = Vec::new();

        while let Some(interval) = heap.pop() {
            if !Self::precludes(&intervals, &interval, rest) {
                intervals.push(interval);
            }
        }

        Self::restrict_to_actual_intervals(
            &mut intervals,
            f32::from(duration) + f32::from(rest),
            count,
        );
        intervals
    }

    pub fn analyze(&self, duration: u8, rest: u8, count: u8, tod: bool) {
        let intervals = self.intervals(duration, rest, count);
        self.dump(&intervals, tod);

        if intervals.len() != usize::from(count) {
            panic!(
                "Was told to find {} intervals, but found {}",
                count,
                intervals.len()
            );
        }
    }

    pub fn already_has_meters_per_second(&mut self) -> bool {
        self.trkpts.iter().all(|t| t.meters_per_second.is_some())
    }

    pub fn fill_in_meters_per_second(&mut self) {
        let mut iter = self.trkpts.iter_mut();
        if let Some(&mut Trkpt {
            mut lat,
            mut lon,
            mut time,
            mut elevation_meters,
            ..
        }) = iter.next()
        {
            for trkpt in iter {
                let new_lat = trkpt.lat;
                let new_lon = trkpt.lon;
                let new_time = trkpt.time;
                let new_elevation_meters = trkpt.elevation_meters;
                let duration =
                    ((new_time - time).num_microseconds().unwrap() as f64) / 1_000_000.00;
                let length_2d = Haversine.length(&LineString::<f64>::from(vec![
                    (lon, lat),
                    (new_lon, new_lat),
                ]));
                let length_3d = match (new_elevation_meters, elevation_meters) {
                    (Some(em1), Some(em2)) => (length_2d.powi(2) + (em1 - em2).powi(2)).sqrt(),
                    _ => length_2d,
                };

                let candidate = length_3d / duration;
                trkpt.meters_per_second = Some(if candidate.is_nan() { 0.0 } else { candidate });
                lat = new_lat;
                lon = new_lon;
                time = new_time;
                elevation_meters = new_elevation_meters;
            }
        }
    }
}

impl FromStr for Gpx {
    type Err = roxmltree::Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        Ok(Gpx {
            trkpts: Self::trkpts(&Document::parse(string)?),
        })
    }
}
