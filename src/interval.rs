use nom_fun::duration::Duration;

use nom::types::CompleteStr;

// mod interval_parse;

pub fn average_from_string(content: &str) -> Option<Duration> {
    let pairs = nom_fun::interval_parse::many_pace_duration_pairs(CompleteStr(&content))
        .unwrap()
        .1;
    // For now we ignore the duration, which is typically 75 seconds
    let total: Duration = pairs.iter().map(|(pace, _duration)| pace).sum();

    if pairs.is_empty() {
        None
    } else {
        Some(total / pairs.len() as u32)
    }
}
