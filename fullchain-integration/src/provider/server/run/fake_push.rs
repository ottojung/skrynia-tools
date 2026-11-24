//! Handle for the disposable TLS provider thread and its captured requests.
use super::Seen;
use crate::Materials;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

pub struct FakePush {
    pub addr: SocketAddr,
    pub ca_der: Vec<u8>,
    seen: Arc<Mutex<Vec<Seen>>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
    _m: Arc<Materials>,
}
impl FakePush {
    pub fn start(plan: super::ReplyPlan) -> Self {
        super::accept::start(plan)
    }
    pub fn seen(&self) -> Vec<Seen> {
        self.seen.lock().unwrap().clone()
    }
    pub(crate) fn parts(
        addr: SocketAddr,
        ca_der: Vec<u8>,
        seen: Arc<Mutex<Vec<Seen>>>,
        stop: Arc<AtomicBool>,
        worker: JoinHandle<()>,
        m: Arc<Materials>,
    ) -> Self {
        Self {
            addr,
            ca_der,
            seen,
            stop,
            worker: Some(worker),
            _m: m,
        }
    }
}
impl Drop for FakePush {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(w) = self.worker.take() {
            let _ = w.join();
        }
    }
}
