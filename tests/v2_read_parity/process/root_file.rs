use std::{fs, path::Path};
pub fn credentials(dir: &Path) -> Result<(String, String), String> {
    let mut root = None;
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_file() {
            return Err("unexpected entry in grant fixture".into());
        }
        let bytes = fs::read(entry.path()).map_err(|e| e.to_string())?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        if value.get("parent").is_some_and(serde_json::Value::is_null) {
            if root.is_some() {
                return Err("multiple root grants".into());
            }
            let id = value["id"].as_str().ok_or("missing root id")?;
            let token = value["rootIssuerToken"]
                .as_str()
                .ok_or("missing root issuer")?;
            root = Some((id.to_owned(), token.to_owned()));
        }
    }
    root.ok_or_else(|| "missing root grant".into())
}
#[cfg(test)]
mod tests {
    use super::credentials;
    use std::fs;
    #[test]
    fn selects_root_independently_of_grant_filename() {
        let path = std::env::temp_dir().join(format!("skrynia-v2-root-{}", std::process::id()));
        fs::create_dir(&path).unwrap();
        fs::write(path.join("0-child"), br#"{"id":"child","parent":"root"}"#).unwrap();
        fs::write(
            path.join("z-root"),
            br#"{"id":"root","parent":null,"rootIssuerToken":"issuer"}"#,
        )
        .unwrap();
        assert_eq!(
            credentials(&path).unwrap(),
            ("root".into(), "issuer".into())
        );
        fs::remove_dir_all(path).unwrap();
    }
}
