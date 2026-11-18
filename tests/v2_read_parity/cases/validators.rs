use super::*;
use sha2::{Digest, Sha256};
pub fn verify(
    client: &reqwest::blocking::Client,
    obj: &MemoryStore,
    url: &str,
) -> Result<(), String> {
    let tag = format!("\"{}\"", hex::encode(Sha256::digest(b"abcdef")));
    let tests = [
        ("If-None-Match", tag.clone()),
        ("If-None-Match", format!("W/{tag}")),
        ("If-None-Match", format!("\"other\", {tag}")),
        ("If-None-Match", "\"not-equal\"".into()),
        ("If-Range", tag.clone()),
        ("If-Range", format!("W/{tag}")),
        ("If-Range", "\"different\"".into()),
    ];
    for (header, value) in tests {
        super::compare::one(client, obj, url, "GET", "/store/a.txt", header, &value)?;
    }
    Ok(())
}
