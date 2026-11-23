//! Signs the leaf CSR and emits its DER certificate and PKCS#8 key.
use super::cert_ossl::{ossl, p};
use std::path::Path;

pub fn make(dir: &Path) {
    let (cak, cap, csr, ext) = (
        p(dir, "ca.key"),
        p(dir, "ca.pem"),
        p(dir, "s.csr"),
        p(dir, "s.ext"),
    );
    let (sp, sd, sk, pk8) = (
        p(dir, "s.pem"),
        p(dir, "s.der"),
        p(dir, "s.key"),
        p(dir, "s.pk8"),
    );
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
