//! Correct Host/SNI/cert and all required WebPush headers on a live 201.
use super::probe::{header, request, seen, send, service};
use super::{HOST, ReplyPlan};
use base64ct::{Base64UrlUnpadded, Encoding};

pub fn happy() {
    let svc = service(ReplyPlan::status(201));
    let req = request(svc.addr.ip(), 443);
    assert_eq!(send(&svc, &req, 2_000).unwrap().status, 201);
    let seen = seen(&svc);
    assert_eq!(seen.len(), 1);
    let one = &seen[0];
    let topic = Base64UrlUnpadded::encode_string(&req.topic);
    assert_eq!(one.method, "POST");
    assert_eq!(one.target, "/push/x");
    assert_eq!(one.host, HOST);
    assert_eq!(one.sni.as_deref(), Some(HOST));
    assert_eq!(
        header(one, "authorization"),
        Some(req.authorization.as_str())
    );
    assert_eq!(header(one, "content-encoding"), Some("aes128gcm"));
    assert_eq!(header(one, "ttl"), Some("60"));
    assert_eq!(header(one, "topic"), Some(topic.as_str()));
    assert_eq!(one.body, req.body);
    assert!(one.headers.iter().all(|(name, _)| matches!(
        name.as_str(),
        "host"
            | "authorization"
            | "content-encoding"
            | "content-type"
            | "ttl"
            | "topic"
            | "content-length"
            | "accept"
            | "user-agent"
    )));
}
