use {
    chrono_tz::Tz::{self, America__Denver},
    std::sync::OnceLock,
};

pub mod fit;
pub mod gpx;
pub mod interval_parse;
pub mod kml;
pub mod misc;
pub mod tcx;
pub mod xlsx;

static TZ: OnceLock<Tz> = OnceLock::new();

// Must be called before tz() can be called. I.e., it's meant to be
// called as soon as the options have been parsed.
pub fn set_tz(tz: Option<Tz>) {
    let _ = TZ.set(tz.unwrap_or_else(|| {
        iana_time_zone::get_timezone()
            .unwrap_or_else(|_| "America/Denver".to_string())
            .parse()
            .unwrap_or(America__Denver)
    }));
}

pub(crate) fn tz() -> &'static Tz {
    TZ.get().unwrap()
}
