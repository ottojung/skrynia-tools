//! openssl-backed disposable certificate generation primitives.
use std::path::Path;
use std::process::Command;

pub(super) fn ossl(args: &[&str]) {
    let s = Command::new("openssl")
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .expect("openssl");
    assert!(s.success(), "openssl {:?}", args);
}
pub(super) fn p(dir: &Path, n: &str) -> String {
    dir.join(n).to_string_lossy().into_owned()
}
pub fn make(dir: &Path, host: &str) {
    super::cert_ca::make_ca(dir);
    super::cert_leaf::make_leaf(dir, host);
}
