use std::{cell::Cell, cell::RefCell, collections::HashMap, net::IpAddr};
use webpush_sender::{
    error::IoFailure,
    push::{PushRequest, PushTransport, Reply},
    sub::Resolver,
};

pub struct ScriptedTransport {
    pub statuses: RefCell<Vec<u16>>,
    pub calls: RefCell<Vec<PushRequest>>,
    pub retry_after_ms: Cell<Option<u64>>,
}

impl ScriptedTransport {
    pub fn new(statuses: Vec<u16>) -> Self {
        Self {
            statuses: RefCell::new(statuses),
            calls: RefCell::new(Vec::new()),
            retry_after_ms: Cell::new(None),
        }
    }
}

impl PushTransport for ScriptedTransport {
    fn send(&self, req: &PushRequest) -> core::result::Result<Reply, IoFailure> {
        self.calls.borrow_mut().push(req.clone());
        let status = self.statuses.borrow_mut().remove(0);
        Ok(Reply {
            status,
            retry_after_ms: self.retry_after_ms.get(),
        })
    }
}

pub struct ScriptedResolver {
    pub answers: RefCell<HashMap<String, Vec<IpAddr>>>,
}

impl ScriptedResolver {
    pub fn set(&self, host: &str, ip: &str) {
        self.answers
            .borrow_mut()
            .insert(host.to_string(), vec![ip.parse().unwrap()]);
    }
}

impl Resolver for ScriptedResolver {
    fn resolve(&self, host: &str) -> core::result::Result<Vec<IpAddr>, IoFailure> {
        self.answers.borrow().get(host).cloned().ok_or(IoFailure)
    }
}
