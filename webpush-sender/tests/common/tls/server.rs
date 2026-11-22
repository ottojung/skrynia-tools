//! Owns one ephemeral loopback listener and its disposable CA.
use super::Seen;
use super::accept;
use super::certs::Materials;
use super::config::ReplyPlan;
use std::net::{SocketAddr, TcpListener};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

pub struct FakePushService {
    pub addr: SocketAddr,
    pub ca_der: Vec<u8>,
    pub seen: Arc<Mutex<Vec<Seen>>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl FakePushService {
    pub fn start(host: &str, plan: ReplyPlan) -> Self {
        let materials = Materials::new(host);
        let ca_der = materials.ca_der.clone();
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        listener.set_nonblocking(true).unwrap();
        let addr = listener.local_addr().unwrap();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let (m, p, s, t) = (
            Arc::new(materials),
            Arc::new(plan),
            seen.clone(),
            stop.clone(),
        );
        let worker = std::thread::spawn(move || accept::run(listener, m, p, s, t));
        Self {
            addr,
            ca_der,
            seen,
            stop,
            worker: Some(worker),
        }
    }
}

impl Drop for FakePushService {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
