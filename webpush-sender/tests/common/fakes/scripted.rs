use std::cell::{Cell, RefCell};
use webpush_sender::{
    error::IoFailure,
    push::{PushRequest, PushTransport, Reply},
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
