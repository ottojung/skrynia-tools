//! Disposable CA and server certificate stored in a unique temp directory.
mod leaf;
mod openssl;
mod run;

use std::fs;
use std::path::PathBuf;
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
        let dir = std::env::temp_dir().join(format!("wps-fake-{}-{}", std::process::id(), id));
        fs::create_dir_all(&dir).unwrap();
        let ca = openssl::ca(&dir);
        let server = leaf::make(&dir, host, &ca);
        Self {
            ca_der: openssl::read(&ca.der),
            cert_der: openssl::read(&server.0),
            key_pk8: openssl::read(&server.1),
            dir,
        }
    }
}

impl Drop for Materials {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}
