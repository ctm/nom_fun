extern crate nom_fun;

use {
    digital_duration_nom::duration::Duration,
    nom_fun::{gpx::Gpx, misc},
    std::{io::Result, path::PathBuf},
    structopt::StructOpt,
};

pub fn main() -> Result<()> {
    let opt = Opt::from_args();

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
                let gpx = Gpx::from_string(&contents);
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
    let pairs = nom_fun::interval_parse::many_pace_duration_pairs(&content)
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

#[derive(StructOpt, Debug)]
#[structopt()]
struct Opt {
    /// Duration (seconds) of each interval
    #[structopt(short = "d", long = "interval-duration", default_value = "75")]
    pub interval_duration: u8,
    /// Seconds of rest between intervals
    #[structopt(short = "r", long = "interval-rest", default_value = "30")]
    pub interval_rest: u8,
    /// Number of intervals to find within file
    #[structopt(short = "c", long = "interval-count", default_value = "12")]
    pub interval_count: u8,
    #[structopt(parse(from_os_str))]
    pub files: Vec<PathBuf>,
}
