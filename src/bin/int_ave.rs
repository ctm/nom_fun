extern crate nom_fun;

use {
    chrono_tz::Tz,
    clap::Parser,
    digital_duration_nom::duration::Duration,
    nom_fun::{gpx::Gpx, misc},
    std::{
        io::{self, Result},
        path::PathBuf,
        str::FromStr,
    },
};

pub fn main() -> Result<()> {
    let opt = Opt::parse();
    nom_fun::set_tz(opt.time_zone);

    for path in opt.files {
        let contents = misc::contents_from(&path)?;
        match path.extension().map(std::ffi::OsStr::to_str) {
            None => {
                if let Some(average) = average_from_string(&contents) {
                    println!("Average: {:.1}", average);
                }
            }
            Some(None) => println!("Non-UTF8 extension"),
            Some(Some("fit")) => println!("FIT"),
            Some(Some("gpx")) => {
                let mut gpx = Gpx::from_str(&contents).map_err(io::Error::other)?;
                if gpx.already_has_meters_per_second() {
                    println!("Old:");
                    gpx.analyze(opt.interval_duration, opt.interval_rest, opt.interval_count);
                    println!("New:");
                }
                gpx.fill_in_meters_per_second();
                // println!("{:?}", gpx);
                gpx.analyze(opt.interval_duration, opt.interval_rest, opt.interval_count);
            }
            Some(Some("kml")) => println!("KML"),
            Some(Some("tcx")) => println!("TCX"),
            Some(Some("xlsx")) => println!("XLSX"),
            Some(Some(extension)) => println!("Unknown extension {}", extension),
        }
    }
    Ok(())
}

fn average_from_string(content: &str) -> Option<Duration> {
    let pairs = nom_fun::interval_parse::many_pace_duration_pairs(content)
        .unwrap()
        .1;
    // For now we ignore the duration, which is typically 75 seconds
    let total: Duration = pairs.iter().map(|(pace, _duration)| pace).sum();

    if pairs.is_empty() {
        None
    } else {
        Some(total / pairs.len() as u32)
    }
}

#[derive(Parser, Debug)]
struct Opt {
    /// Duration (seconds) of each interval
    #[arg(short = 'd', long = "interval-duration", default_value = "75")]
    pub interval_duration: u8,
    /// Seconds of rest between intervals
    #[arg(short = 'r', long = "interval-rest", default_value = "30")]
    pub interval_rest: u8,
    /// Number of intervals to find within file
    #[arg(short = 'c', long = "interval-count", default_value = "12")]
    pub interval_count: u8,
    #[arg(short, long)]
    pub time_zone: Option<Tz>,
    #[arg()]
    pub files: Vec<PathBuf>,
}
