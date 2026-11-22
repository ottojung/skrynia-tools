//! Outbound transport boundary. Production injects a real HTTPS client bound
//! to `req.ip`; tests use an in-memory fake endpoint.
use super::{IoFailure, PushRequest};
use std::cell::RefCell;

pub struct Reply {
    pub status: u16,
    pub retry_after_ms: Option<u64>,
}

pub trait PushTransport {
    fn send(&self, req: &PushRequest) -> core::result::Result<Reply, IoFailure>;
}

pub struct FakeTransport {
    pub reply: Reply,
    calls: RefCell<Vec<PushRequest>>,
}

impl FakeTransport {
    pub fn new(status: u16) -> Self {
        Self {
            reply: Reply {
                status,
                retry_after_ms: None,
            },
            calls: RefCell::new(Vec::new()),
        }
    }

    pub fn requests(&self) -> Vec<PushRequest> {
        self.calls.borrow().clone()
    }
}

impl PushTransport for FakeTransport {
    fn send(&self, req: &PushRequest) -> core::result::Result<Reply, IoFailure> {
        self.calls.borrow_mut().push(clone_request(req));
        Ok(Reply {
            status: self.reply.status,
            retry_after_ms: self.reply.retry_after_ms,
        })
    }
}

fn clone_request(req: &PushRequest) -> PushRequest {
    PushRequest {
        host: req.host.clone(),
        ip: req.ip,
        port: req.port,
        path: req.path.clone(),
        authorization: req.authorization.clone(),
        ttl: req.ttl,
        topic: req.topic,
        body: req.body.clone(),
    }
}
