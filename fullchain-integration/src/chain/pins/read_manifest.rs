use super::NAMES;
use std::collections::BTreeMap;
use std::path::Path;
use std::{fs, io};

fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid artifact pin manifest")
}

pub(super) fn load(art: &Path) -> io::Result<BTreeMap<String, String>> {
    let manifest = fs::read_to_string(art.join("SHA256SUMS"))?;
    let mut pins = BTreeMap::new();
    for line in manifest.lines() {
        let (hash, name) = line.split_once("  ").ok_or_else(invalid)?;
        if !NAMES.contains(&name)
            || hash.len() != 64
            || !hash
                .bytes()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
            || pins.insert(name.to_owned(), hash.to_owned()).is_some()
        {
            return Err(invalid());
        }
    }
    if pins.len() != NAMES.len() {
        return Err(invalid());
    }
    Ok(pins)
}
