//! Thin openssl CLI wrapper for certificate generation.
use std::fs;
use std::path::Path;

pub use super::run::run;

pub struct Paths {
    pub key: String,
    pub pem: String,
    pub der: String,
}

pub fn ca(dir: &Path) -> Paths {
    let p = |n: &str| dir.join(n).to_str().unwrap().to_owned();
    let out = Paths {
        key: p("ca.key"),
        pem: p("ca.pem"),
        der: p("ca.der"),
    };
    let args = vec![
        "req",
        "-x509",
        "-newkey",
        "rsa:2048",
        "-nodes",
        "-days",
        "1",
        "-keyout",
        &out.key,
        "-out",
        &out.pem,
        "-subj",
        "/CN=Fake Test CA",
        "-addext",
        "basicConstraints=critical,CA:TRUE",
        "-addext",
        "keyUsage=critical,keyCertSign,cRLSign",
    ];
    run(args);
    run(vec![
        "x509", "-in", &out.pem, "-outform", "DER", "-out", &out.der,
    ]);
    out
}

pub fn read(path: &str) -> Vec<u8> {
    fs::read(path).unwrap()
}
