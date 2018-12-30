#[derive(Debug)]
pub struct Gpx<'a> {
    doc: roxmltree::Document<'a>,
}

impl<'a> Gpx<'a> {
    pub fn from_string(string:&'a String) -> Self {
        let doc = roxmltree::Document::parse(string).unwrap();
        Gpx { doc }
    }
}
