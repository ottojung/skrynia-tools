//! Disposable server leaf certificate signed by the local test CA.
use super::cert_gen::{ossl, p};
use std::{fs, path::Path};

pub(super) fn make_leaf(dir: &Path, host: &str) {
    let (cak, cap) = (p(dir, "ca.key"), p(dir, "ca.pem"));
    let (sk, csr, sp, sd, pk8, ext) = (
        p(dir, "s.key"),
        p(dir, "s.csr"),
        p(dir, "s.pem"),
        p(dir, "s.der"),
        p(dir, "s.pk8"),
        p(dir, "s.ext"),
    );
    fs::write(&ext, format!("subjectAltName=DNS:{host}\nbasicConstraints=CA:FALSE\nkeyUsage=digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\n")).unwrap();
    let subj = format!("/CN={host}");
    ossl(&[
        "req", "-new", "-newkey", "rsa:2048", "-nodes", "-keyout", &sk, "-out", &csr, "-subj",
        &subj,
    ]);
    ossl(&[
        "x509",
        "-req",
        "-in",
        &csr,
        "-CA",
        &cap,
        "-CAkey",
        &cak,
        "-CAcreateserial",
        "-days",
        "1",
        "-extfile",
        &ext,
        "-out",
        &sp,
    ]);
    ossl(&["x509", "-in", &sp, "-outform", "DER", "-out", &sd]);
    ossl(&[
        "pkcs8", "-topk8", "-nocrypt", "-in", &sk, "-outform", "DER", "-out", &pk8,
    ]);
}
