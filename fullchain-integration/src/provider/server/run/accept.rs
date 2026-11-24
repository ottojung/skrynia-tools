//! Binds a disposable TLS listener and drives the accept loop.
use super::{FakePush, ReplyPlan, Seen};
use crate::Materials;
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener as TokioListener;
use tokio::runtime::Builder;

pub fn start(plan: ReplyPlan) -> FakePush {
    let m = Arc::new(Materials::new("push.example.test"));
    let ca_der = m.ca_der.clone();
    let l = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    l.set_nonblocking(true).unwrap();
    let addr = l.local_addr().unwrap();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let stop = Arc::new(AtomicBool::new(false));
    let (m2, p, s, t) = (m.clone(), Arc::new(plan), seen.clone(), stop.clone());
    let worker = std::thread::spawn(move || {
        Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(serve(l, m2, p, s, t));
    });
    FakePush::parts(addr, ca_der, seen, stop, worker, m)
}
async fn serve(
    l: TcpListener,
    m: Arc<Materials>,
    p: Arc<ReplyPlan>,
    s: Arc<Mutex<Vec<Seen>>>,
    stop: Arc<AtomicBool>,
) {
    let l = TokioListener::from_std(l).unwrap();
    while !stop.load(Ordering::Relaxed) {
        let Ok(Ok((stream, _))) =
            tokio::time::timeout(std::time::Duration::from_millis(200), l.accept()).await
        else {
            continue;
        };
        let (m, p, s) = (m.clone(), p.clone(), s.clone());
        tokio::spawn(async move {
            super::accept_handle::handle(stream, m, p, s).await;
        });
    }
}
