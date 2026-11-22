//! 3xx is not followed; 2xx/4xx/5xx are surfaced as exact status codes.
use super::ReplyPlan;
use super::probe::{request, seen, send, service};

pub fn redirects_and_status() {
    let redirect = service(ReplyPlan::redirect("https://elsewhere.test/next"));
    let req = request(redirect.addr.ip(), 443);
    assert_eq!(send(&redirect, &req, 2_000).unwrap().status, 302);
    assert_eq!(seen(&redirect).len(), 1);
    for status in [201, 403, 429, 500] {
        let svc = service(ReplyPlan::status(status));
        let req = request(svc.addr.ip(), 443);
        assert_eq!(send(&svc, &req, 2_000).unwrap().status, status);
    }
}
