extern crate nom_fun;

mod interval;

use std::path::Path;
use std::fs;
use std::io::Read;

use nom_fun::gpx::Gpx;

// TODO: error handling
fn contents_from(path: &Path) -> String {
    let mut contents = String::new();
    let mut file = fs::File::open(path).unwrap();
    file.read_to_string(&mut contents).unwrap();

    contents
}

pub fn main() {
    for filename in std::env::args().skip(1) {
        let path = Path::new(&filename);
        let contents = contents_from(&path);
        match path.extension().map(|extension| extension.to_str()) {
            None => {
                println!("Average: {:.1}", interval::average_from_string(&contents));
            }
            Some(None) => println!("Non-UTF8 extension"),
            Some(Some("fit")) => println!("FIT"),
            Some(Some("gpx")) => {
                let gpx = Gpx::from_string(&contents);
                println!("{:?}", gpx);
            },
            Some(Some("kml")) => println!("KML"),
            Some(Some("tcx")) => println!("TCX"),
            Some(Some("xlsx")) => println!("XLSX"),
            Some(Some(extension)) => println!("Unknown extension {}", extension),
        }
    }
}
