// random code that doesn't have a better place to live.

use std::{
    fs,
    io::{Read, Result},
    path::Path,
};

pub fn contents_from(path: &Path) -> Result<String> {
    let mut contents = String::new();
    let mut file = fs::File::open(path)?;
    file.read_to_string(&mut contents)?;

    Ok(contents)
}
