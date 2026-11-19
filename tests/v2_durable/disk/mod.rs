mod fs;
mod restart;
mod uncertain;

pub use fs::Fs;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

pub struct R(pub AtomicU64);
impl skrynia_program0::cap::Random for R {
    type Err = std::convert::Infallible;
    fn fill(&self, out: &mut [u8]) -> Result<(), Self::Err> {
        for b in out {
            *b = self.0.fetch_add(1, Ordering::Relaxed) as u8;
        }
        Ok(())
    }
}

pub fn tmp(name: &str) -> PathBuf {
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("skrynia-durable-{name}-{t}"))
}
