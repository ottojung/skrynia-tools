//! Remaps SSRF-validated global pin to loopback for disposable HTTPS provider.
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Mutex;
use webpush_sender::error::IoFailure;
use webpush_sender::push::{HttpsTransport, PushRequest, PushTransport, Reply};

pub struct LoopbackPush {
    ca_der: Vec<u8>,
    port: u16,
    pub sent: Mutex<Vec<PushRequest>>,
}
impl LoopbackPush {
    pub fn new(ca_der: Vec<u8>, port: u16) -> Self {
        Self {
            ca_der,
            port,
            sent: Mutex::new(Vec::new()),
        }
    }
}
impl PushTransport for LoopbackPush {
    fn send(&self, req: &PushRequest) -> core::result::Result<Reply, IoFailure> {
        self.sent.lock().unwrap().push(req.clone());
        let mut m = req.clone();
        m.ip = IpAddr::V4(Ipv4Addr::LOCALHOST);
        HttpsTransport::for_test(&self.ca_der, self.port, 4_000).send(&m)
    }
}
