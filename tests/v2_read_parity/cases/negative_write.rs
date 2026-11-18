use super::*;
pub fn verify(url: &str) -> Result<(), String> {
    let client = reqwest::blocking::Client::new();
    let backend = MemoryStore;
    for (method, path, header, value) in [
        ("POST", "/store/a.txt", "", ""),
        ("PUT", "/store/a.txt", "", ""),
        ("DELETE", "/store/a.txt", "", ""),
        ("DELETE", "/store/missing", "", ""),
        ("POST", "/store/bad%GG", "", ""),
        ("PUT", "/store/a.txt", "If-Match", "*"),
        ("POST", "/store/a.txt", "X-Skrynia-If-SHA256", "0"),
        ("PUT", "/store/a.txt", "X-Skrynia-If-SHA256", "ZZ"),
        ("DELETE", "/store/a.txt", "If-None-Match", "*"),
    ] {
        super::compare::one(&client, &backend, url, method, path, header, value)?;
    }
    Ok(())
}
