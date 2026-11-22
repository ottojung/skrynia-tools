//! Accept loop for the disposable TLS endpoint (loopback only).
use super::certs::Materials;
use super::config::ReplyPlan;
use super::http;
use super::{Seen, handshake};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::runtime::Builder;
use tokio_rustls::TlsAcceptor;

pub fn run(
    listener: std::net::TcpListener,
    materials: Arc<Materials>,
    plan: Arc<ReplyPlan>,
    seen: Arc<Mutex<Vec<Seen>>>,
    stop: Arc<AtomicBool>,
) {
    let runtime = Builder::new_current_thread().enable_all().build().unwrap();
    runtime.block_on(serve(listener, materials, plan, seen, stop));
}

async fn serve(
    listener: std::net::TcpListener,
    materials: Arc<Materials>,
    plan: Arc<ReplyPlan>,
    seen: Arc<Mutex<Vec<Seen>>>,
    stop: Arc<AtomicBool>,
) {
    let listener = TcpListener::from_std(listener).unwrap();
    while !stop.load(Ordering::Relaxed) {
        let Ok(Ok((stream, _))) =
            tokio::time::timeout(Duration::from_millis(200), listener.accept()).await
        else {
            continue;
        };
        let (m, p, log) = (materials.clone(), plan.clone(), seen.clone());
        tokio::spawn(async move {
            let sni = Arc::new(Mutex::new(None));
            let config = handshake::config(&m, sni.clone());
            let Ok(tls) = TlsAcceptor::from(Arc::new(config)).accept(stream).await else {
                return;
            };
            let mut captured = Vec::new();
            let work = http::exchange(tls, &p, &mut captured, sni.lock().unwrap().clone());
            let _ = tokio::time::timeout(Duration::from_secs(5), work).await;
            if !captured.is_empty() {
                log.lock().unwrap().extend(captured);
            }
        });
    }
}
