//! Ephemeral CA and leaf certificate material for the HTTPS fake provider.
use super::{cert_ca, cert_leaf};
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
        let dir = std::env::temp_dir().join(format!("fc-push-{id}-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        cert_ca::make(&dir);
        cert_leaf::make(&dir, host);
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
