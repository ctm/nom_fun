use std::path::Path;
use std::fs;
use std::io::Read;

// For now we're using unwrap and just letting errors result in a panic

#[derive(Debug)]
pub struct Gpx<'a> {
    doc: roxmltree::Document<'a>,
}

pub fn from_path<'a>(path: &Path, string:&'a mut String) -> Gpx<'a> {
    let mut file = fs::File::open(path).unwrap();
    file.read_to_string(string).unwrap();
    let doc = roxmltree::Document::parse(string).unwrap();
    Gpx { doc }
}
