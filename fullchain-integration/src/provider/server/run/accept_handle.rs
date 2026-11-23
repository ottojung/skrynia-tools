//! One TLS connection: read request, reply per plan, record what was seen.
use super::Seen;
use crate::{Materials, ReplyPlan, read_req, server_config, write_reply};
use std::sync::{Arc, Mutex};
use tokio_rustls::TlsAcceptor;

pub(super) async fn handle(
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
