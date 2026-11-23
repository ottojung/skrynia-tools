use super::types::{FakePush, ReplyPlan, Seen};
use crate::provider::server::http::{read_req, write_reply};
use crate::provider::tls::{Materials, server_config};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener as TokioListener;
use tokio::runtime::Builder;
use tokio_rustls::TlsAcceptor;

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
            handle(stream, m, p, s).await;
        });
    }
}
async fn handle(
    stream: tokio::net::TcpStream,
    m: Arc<Materials>,
    p: Arc<ReplyPlan>,
    s: Arc<Mutex<Vec<Seen>>>,
) {
    let sni = Arc::new(Mutex::new(None));
    let cfg = server_config(&m, sni.clone());
    let Ok(mut tls) = TlsAcceptor::from(Arc::new(cfg)).accept(stream).await else {
        return;
    };
    let Ok((head, body)) = read_req(&mut tls).await else {
        return;
    };
    let _ = write_reply(&mut tls, p.status, &p.headers, &p.body).await;
    let mut seen = parse(&head, body);
    seen.sni = sni.lock().unwrap().clone();
    s.lock().unwrap().push(seen);
}
fn parse(head: &str, body: Vec<u8>) -> Seen {
    let mut lines = head.lines();
    let mut parts = lines.next().unwrap_or_default().split_whitespace();
    let (method, target) = (
        parts.next().unwrap_or("").to_owned(),
        parts.next().unwrap_or("").to_owned(),
    );
    let mut headers = Vec::new();
    for line in lines {
        if let Some((n, v)) = line.split_once(':') {
            headers.push((n.trim().to_ascii_lowercase(), v.trim().to_owned()));
        }
    }
    let host = headers
        .iter()
        .find(|(n, _)| n == "host")
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    Seen {
        method,
        target,
        host,
        sni: None,
        headers,
        body,
    }
}
