//! A foreign CA and a hostname outside the cert SAN both fail closed.
use super::probe::{request, seen, service};
use super::{FakePushService, OTHER_HOST, ReplyPlan};
use webpush_sender::push::{HttpsTransport, PushTransport};

pub fn wrong_trust() {
    let good = service(ReplyPlan::status(201));
    let other = FakePushService::start(OTHER_HOST, ReplyPlan::status(201));
    let ca = other.ca_der.clone();
    let bad = HttpsTransport::for_test(&ca, good.addr.port(), 2_000);
    assert!(bad.send(&request(good.addr.ip(), 443)).is_err());
    assert!(seen(&good).is_empty());
    let mut renamed = request(good.addr.ip(), 443);
    renamed.host = OTHER_HOST.into();
    let mismatch = HttpsTransport::for_test(&good.ca_der, good.addr.port(), 2_000);
    assert!(mismatch.send(&renamed).is_err());
}
