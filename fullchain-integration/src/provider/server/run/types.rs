use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

#[derive(Debug, Clone, Default)]
pub struct Seen {
    pub method: String,
    pub target: String,
    pub host: String,
    pub sni: Option<String>,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}
impl Seen {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}
#[derive(Clone)]
pub struct ReplyPlan {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}
impl ReplyPlan {
    pub fn status(status: u16) -> Self {
        Self {
            status,
            headers: Vec::new(),
            body: b"ok".to_vec(),
        }
    }
    pub fn retry(mut self, secs: u64) -> Self {
        self.headers.push(("Retry-After".into(), secs.to_string()));
        self
    }
}
pub struct FakePush {
    pub addr: SocketAddr,
    pub ca_der: Vec<u8>,
    seen: Arc<Mutex<Vec<Seen>>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
    _m: Arc<crate::provider::tls::Materials>,
}
impl FakePush {
    pub fn start(plan: ReplyPlan) -> Self {
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
        m: Arc<crate::provider::tls::Materials>,
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
