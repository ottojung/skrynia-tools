use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
static SEQ: AtomicU64 = AtomicU64::new(0);

pub struct Materials {
    pub ca_der: Vec<u8>,
    pub cert_der: Vec<u8>,
    pub key_pk8: Vec<u8>,
    dir: PathBuf,
}
impl Materials {
    pub fn new(host: &str) -> Self {
        let id = SEQ.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("fc-push-{id}-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        make(&dir, host);
        Self {
            ca_der: fs::read(dir.join("ca.der")).unwrap(),
            cert_der: fs::read(dir.join("s.der")).unwrap(),
            key_pk8: fs::read(dir.join("s.pk8")).unwrap(),
            dir,
        }
    }
}
impl Drop for Materials {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}
fn ossl(args: &[&str]) {
    let s = Command::new("openssl")
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .expect("openssl");
    assert!(s.success(), "openssl {:?}", args);
}
fn p(dir: &Path, n: &str) -> String {
    dir.join(n).to_string_lossy().into_owned()
}
fn make(dir: &Path, host: &str) {
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
