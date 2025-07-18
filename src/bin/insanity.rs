// Toy program to generate some stats concerning Elevation Insanity.
// Mostly I wrote this to recover the transition times.

use {
    chrono::{DateTime, Duration, Utc},
    clap::Parser,
    nom::{
        IResult, Parser as _,
        bytes::complete::tag,
        character::complete::one_of,
        combinator::{all_consuming, map},
        sequence::terminated,
    },
    nom_fun::{gpx::Gpx, misc},
    roxmltree::Document,
    std::{
        error::Error,
        fmt::{self, Display, Formatter},
        path::PathBuf,
        result,
        str::FromStr,
    },
};

fn to_str(duration: &Duration) -> String {
    let hours = duration.num_hours();
    let total_secs = duration.num_seconds() % (60 * 60);
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;
    format!("{}:{:02}:{:02}", hours, minutes, seconds)
}

fn alternate_stop(
    start: &DateTime<Utc>,
    durations: &[DurationOverride],
    i: u8,
) -> Option<DateTime<Utc>> {
    for duration in durations {
        if duration.hike_index == i {
            return Some(*start + duration.duration);
        }
    }
    None
}

fn main() {
    let opt = Opt::parse();
    let durations = &opt.durations;

    let mut start_stops = opt.files.into_iter().enumerate().map(|(i, path)| {
        let contents = misc::contents_from(&path).unwrap();
        let document = Document::parse(&contents).unwrap();
        let mut iter = Gpx::trkpt_iterator(&document);
        let start_trkpt = iter.next().expect("no starting trackpoint");
        let stop_trkpt = iter.last().expect("no stopping trackpoint");
        let start = start_trkpt.time;
        let stop = alternate_stop(&start, durations, i as u8).unwrap_or(stop_trkpt.time);
        (start, stop)
    });

    let mut last_stop = start_stops.next().unwrap().1;

    let mut transitions = start_stops.map(|(start, stop)| {
        let transition = start - last_stop;
        last_stop = stop;
        transition
    });

    print!("{}", to_str(&transitions.next().unwrap()));
    for transition in transitions {
        print!(" {}", to_str(&transition));
    }
    println!();
}

// target/release/insanity --duration=4/3:46:49 ~/Downloads/EI_IX_gpx/*.gpx

#[derive(Parser, Debug)]
struct Opt {
    /// Override a 1-based hike's duration, e.g., --duration=4/3:46:49
    #[arg(long = "duration", value_parser = DurationOverride::from_str)]
    pub durations: Vec<DurationOverride>,
    pub files: Vec<PathBuf>,
}

#[derive(Clone, Debug)]
struct DurationOverride {
    hike_index: u8, // 0-based
    duration: Duration,
}

#[derive(Debug)]
struct ParseDurationOverrideError(());

impl Error for ParseDurationOverrideError {}

impl Display for ParseDurationOverrideError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Can't parse duration override")
    }
}

use digital_duration_nom::duration::duration_parser;

fn duration_override(input: &str) -> IResult<&str, DurationOverride> {
    all_consuming(map(
        (terminated(one_of("123456"), tag("/")), duration_parser),
        |(digit, duration)| {
            let hike_index = digit as u8 - b'1';
            let duration: std::time::Duration = duration.into();
            let duration = Duration::from_std(duration).unwrap();

            DurationOverride {
                duration,
                hike_index,
            }
        },
    ))
    .parse(input)
}

impl FromStr for DurationOverride {
    type Err = ParseDurationOverrideError;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        match duration_override(s) {
            Ok((_, duration_override)) => Ok(duration_override),
            _ => Err(ParseDurationOverrideError(())),
        }
    }
}
