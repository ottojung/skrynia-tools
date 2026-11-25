//! Decode claims from a VAPID JWT and scan source for forbidden patterns.
use base64ct::{Base64UrlUnpadded, Encoding};
use std::{fs, path::Path};

pub fn jwt_aud(header: &str) -> String {
    let jwt = header
        .split_once("vapid t=")
        .unwrap()
        .1
        .split(", k=")
        .next()
        .unwrap();
    let raw = Base64UrlUnpadded::decode_vec(jwt.split('.').nth(1).unwrap()).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    value["aud"].as_str().unwrap().to_owned()
}

pub fn jwt_claims(jwt: &str) -> serde_json::Value {
    let raw = Base64UrlUnpadded::decode_vec(jwt.split('.').nth(1).unwrap()).unwrap();
    serde_json::from_slice(&raw).unwrap()
}

pub fn scan(dir: &Path, needles: &[&str], found: &mut Vec<String>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            scan(&path, needles, found);
        } else if path.extension().is_some_and(|e| e == "rs") {
            let text = fs::read_to_string(&path).unwrap();
            for needle in needles {
                if text.contains(needle) {
                    found.push(format!("{}:{needle}", path.display()));
                }
            }
        }
    }
}

pub fn config_err_redacted() {
    let d = std::env::temp_dir().join(format!("rt-r-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    let p = d.join("bad.json");
    std::fs::write(&p, b"{not json").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    let Err(e) = webpush_sender::push::proc::RuntimeConfig::from_path(&p) else {
        panic!("err")
    };
    assert!(!format!("{e:?}").contains("not json"));
    let _ = std::fs::remove_dir_all(&d);
}
