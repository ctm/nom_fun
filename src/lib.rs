#[macro_use] extern crate nom;
#[macro_use] extern crate custom_derive;
#[macro_use] extern crate newtype_derive;
extern crate time;

pub mod fit;
pub mod gpx;
pub mod kml;
pub mod tcx;
pub mod xlsx;
pub mod interval_parse;
pub mod duration;

// use fit::fit_crc_calc16;

// TODO
