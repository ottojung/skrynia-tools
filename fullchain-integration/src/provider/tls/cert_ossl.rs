//! OpenSSL process helper and path names for disposable certificates.
use std::path::Path;
use std::process::Command;

pub fn ossl(args: &[&str]) {
    let s = Command::new("openssl")
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .expect("openssl");
    assert!(s.success(), "openssl {:?}", args);
}

pub fn p(dir: &Path, n: &str) -> String {
    dir.join(n).to_string_lossy().into_owned()
}
