use super::*;
pub fn verify(url: &str) -> Result<(), String> {
    let client = reqwest::blocking::Client::new();
    let obj = MemoryStore;
    for (method, path, header, value) in [
        ("GET", "/store/a.txt", "", ""),
        ("HEAD", "/store/a.txt", "", ""),
        ("GET", "/store/a.txt", "Range", "bytes=1-3"),
        ("HEAD", "/store/a.txt", "Range", "bytes=1-3"),
        ("GET", "/store/a.txt", "Range", "bytes=-2"),
        ("GET", "/store/a.txt", "Range", "bytes=50-"),
        ("GET", "/store/a.txt", "Range", "bytes=0-1,4-5"),
        ("GET", "/store/a.txt", "If-None-Match", "*"),
        ("GET", "/store/missing", "", ""),
    ] {
        super::compare::one(&client, &obj, url, method, path, header, value)?;
    }
    super::validators::verify(&client, &obj, url)?;
    super::negative_write::verify(url)?;
    Ok(())
}
