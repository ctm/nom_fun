extern crate nom_fun;

use std::path::Path;
use std::ffi::OsStr;

pub fn main() {
    for filename in std::env::args().skip(1) {
        let path = Path::new(&filename);
        match path.extension().map(|extension| extension.to_str()) {
            None => println!("No extension"),
            Some(None) => println!("Non-UTF8 extension"),
            Some(Some("gpx")) => println!("GPX"),
            Some(Some(extension)) => println!("Unknown extension {}", extension),
        }
    }
}
