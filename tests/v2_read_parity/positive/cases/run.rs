use super::*;
use sha2::Digest;
pub fn run(url: &str, token: &str, store: &RootStore) -> Result<(), String> {
    for (method, body, precondition) in [
        ("POST", "alpha", None),
        ("POST", "alpha", None),
        ("PUT", "beta", None),
        ("PUT", "gamma", Some("00".repeat(32))),
        (
            "PUT",
            "gamma",
            Some(hex::encode(sha2::Sha256::digest(b"beta"))),
        ),
        ("GET", "", None),
        ("DELETE", "", Some("11".repeat(32))),
        (
            "DELETE",
            "",
            Some(hex::encode(sha2::Sha256::digest(b"gamma"))),
        ),
        ("DELETE", "", None),
    ] {
        let mut headers = BTreeMap::new();
        headers.insert("authorization".into(), format!("Bearer {token}"));
        if let Some(hash) = precondition {
            headers.insert("x-skrynia-if-sha256".into(), hash);
        }
        super::compare::compare(
            url,
            store,
            Request {
                method: method.into(),
                path: "/store/parity-test.txt".into(),
                query: None,
                headers,
                body: body.as_bytes().to_vec(),
            },
        )?;
    }
    Ok(())
}
