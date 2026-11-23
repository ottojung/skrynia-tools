//! Completes one TLS handshake, reads one request and writes one reply.
use super::accept_parse::parse;
use super::types::{ReplyPlan, Seen};
use crate::provider::server::http::read_req;
use crate::provider::server::http_reply::write_reply;
use crate::provider::tls::{Materials, server_config};
use std::sync::{Arc, Mutex};
use tokio_rustls::TlsAcceptor;

pub async fn handle(
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
