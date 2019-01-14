extern crate nom_fun;

mod interval;
mod opt;

use crate::opt::Opt;

use std::path::Path;
use std::fs;
use std::io::Read;

use nom_fun::gpx::Gpx;
use structopt::StructOpt;

// TODO: error handling
fn contents_from(path: &Path) -> String {
    let mut contents = String::new();
    let mut file = fs::File::open(path).unwrap();
    file.read_to_string(&mut contents).unwrap();

    contents
}

pub fn main() {
    let opt = Opt::from_args();

    for path in opt.files {
        let contents = contents_from(&path);
        match path.extension().map(|extension| extension.to_str()) {
            None => {
                if let Some(average) = interval::average_from_string(&contents) {
                    println!("Average: {:.1}", average);
                }
            }
            Some(None) => println!("Non-UTF8 extension"),
            Some(Some("fit")) => println!("FIT"),
            Some(Some("gpx")) => {
                let gpx = Gpx::from_string(&contents);
                // println!("{:?}", gpx);
                gpx.analyze(opt.interval_duration, opt.interval_rest,
                            opt.interval_count);
            },
            Some(Some("kml")) => println!("KML"),
            Some(Some("tcx")) => println!("TCX"),
            Some(Some("xlsx")) => println!("XLSX"),
            Some(Some(extension)) => println!("Unknown extension {}", extension),
        }
    }
}
