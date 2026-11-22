//! Shared arrangement helpers for the HTTPS adapter tests.
use super::{FakePushService, HOST, ReplyPlan, Seen};
use std::net::IpAddr;
use webpush_sender::error::IoFailure;
use webpush_sender::push::{HttpsTransport, PushRequest, PushTransport, Reply};

pub fn service(plan: ReplyPlan) -> FakePushService {
    FakePushService::start(HOST, plan)
}

pub fn request(ip: IpAddr, port: u16) -> PushRequest {
    PushRequest {
        host: HOST.into(),
        ip,
        port,
        path: "/push/x".into(),
        authorization: "vapid t=test.jwt, k=public-key".into(),
        ttl: 60,
        topic: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
        body: b"ciphertext".to_vec(),
    }
}

pub fn send(
    service: &FakePushService,
    req: &PushRequest,
    timeout_ms: u64,
) -> Result<Reply, IoFailure> {
    let transport = HttpsTransport::for_test(&service.ca_der, service.addr.port(), timeout_ms);
    transport.send(req)
}

pub fn seen(service: &FakePushService) -> Vec<Seen> {
    service.seen.lock().unwrap().clone()
}

pub fn header<'a>(seen: &'a Seen, name: &str) -> Option<&'a str> {
    seen.headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}
