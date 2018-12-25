extern crate nom_fun;

mod interval;

use std::path::Path;
use nom_fun::gpx::Gpx;

pub fn main() {
    for filename in std::env::args().skip(1) {
        let path = Path::new(&filename);
        match path.extension().map(|extension| extension.to_str()) {
            None => {
                println!("Average: {:.1}", interval::average_from_path(&path));
            }
            Some(None) => println!("Non-UTF8 extension"),
            Some(Some("fit")) => println!("FIT"),
            Some(Some("gpx")) => {
                let mut string = String::new();
                let gpx = Gpx::from_path(&path, &mut string);
                println!("{:?}", gpx);
            },
            Some(Some("kml")) => println!("KML"),
            Some(Some("tcx")) => println!("TCX"),
            Some(Some("xlsx")) => println!("XLSX"),
            Some(Some(extension)) => println!("Unknown extension {}", extension),
        }
    }
}
