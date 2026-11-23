//! Generates the disposable test CA certificate and its DER form.
use super::cert_ossl::{ossl, p};
use std::path::Path;

pub fn make(dir: &Path) {
    let (cak, cap, cad) = (p(dir, "ca.key"), p(dir, "ca.pem"), p(dir, "ca.der"));
    ossl(&[
        "req",
        "-x509",
        "-newkey",
        "rsa:2048",
        "-nodes",
        "-days",
        "1",
        "-keyout",
        &cak,
        "-out",
        &cap,
        "-subj",
        "/CN=FC Test CA",
        "-addext",
        "basicConstraints=critical,CA:TRUE",
        "-addext",
        "keyUsage=critical,keyCertSign,cRLSign",
    ]);
    ossl(&["x509", "-in", &cap, "-outform", "DER", "-out", &cad]);
}
