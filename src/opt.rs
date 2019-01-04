use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt()]
pub struct Opt {
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
