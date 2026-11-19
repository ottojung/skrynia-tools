use crate::cases::support::FsDisk;
use skrynia_program0::v2auth::{self, MutateKind, MutateReq};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
static N: AtomicU64 = AtomicU64::new(0);
fn tmp(name: &str) -> PathBuf {
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let n = N.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("skrynia-p0-v2-{name}-{t}-{n}"))
}
fn sha256(b: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    Sha256::new().chain_update(b).finalize().into()
}
mod case0;
mod case1;
mod case2;
mod case3;
mod case4;
mod case5;
mod case6;
