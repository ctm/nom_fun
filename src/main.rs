extern crate nom_fun;

mod gpx;

use std::path::Path;
use crate::gpx::Gpx;

pub fn main() {
    for filename in std::env::args().skip(1) {
        let path = Path::new(&filename);
        match path.extension().map(|extension| extension.to_str()) {
            None => println!("No extension"),
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
