//! Generates the disposable HTTPS leaf key, CSR, extension file and cert.
use super::cert_leaf_sign;
use super::cert_ossl::{ossl, p};
use std::fs;
use std::path::Path;

pub fn make(dir: &Path, host: &str) {
    let (sk, csr, ext) = (p(dir, "s.key"), p(dir, "s.csr"), p(dir, "s.ext"));
    fs::write(
        &ext,
        format!("subjectAltName=DNS:{host}\nbasicConstraints=CA:FALSE\nkeyUsage=digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\n"),
    )
    .unwrap();
    let subj = format!("/CN={host}");
    ossl(&[
        "req", "-new", "-newkey", "rsa:2048", "-nodes", "-keyout", &sk, "-out", &csr, "-subj",
        &subj,
    ]);
    cert_leaf_sign::make(dir);
}
