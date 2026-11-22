//! Signs one SAN-bound server certificate with the disposable CA.
use super::openssl;
use std::fs;
use std::path::Path;

pub fn make(dir: &Path, host: &str, ca: &openssl::Paths) -> (String, String) {
    let p = |n: &str| dir.join(n).to_str().unwrap().to_owned();
    let (key, csr, pem, der, pk8, ext) = (
        p("s.key"),
        p("s.csr"),
        p("s.pem"),
        p("s.der"),
        p("s.pk8"),
        p("s.ext"),
    );
    let san = format!(
        "subjectAltName=DNS:{host}\nbasicConstraints=CA:FALSE\n\
         keyUsage=digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\n"
    );
    fs::write(&ext, san).unwrap();
    let subject = format!("/CN={host}");
    let csr_args = vec![
        "req", "-new", "-newkey", "rsa:2048", "-nodes", "-keyout", &key, "-out", &csr, "-subj",
        &subject,
    ];
    openssl::run(csr_args);
    let sign = vec![
        "x509",
        "-req",
        "-in",
        &csr,
        "-CA",
        &ca.pem,
        "-CAkey",
        &ca.key,
        "-CAcreateserial",
        "-days",
        "1",
        "-extfile",
        &ext,
        "-out",
        &pem,
    ];
    openssl::run(sign);
    openssl::run(vec!["x509", "-in", &pem, "-outform", "DER", "-out", &der]);
    openssl::run(vec![
        "pkcs8", "-topk8", "-nocrypt", "-in", &key, "-outform", "DER", "-out", &pk8,
    ]);
    (der, pk8)
}
