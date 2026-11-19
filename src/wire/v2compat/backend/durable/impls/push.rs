use super::{DurableBackend, Push, RandomSource, Request, Response, StateStore, reply};

impl<S: StateStore, R: RandomSource> Push for DurableBackend<S, R> {
    fn push_subscribe(&self, req: &Request) -> Response {
        super::push_sub::run(self, req).unwrap_or_else(|e| reply::error(e.status, &e.code))
    }
    fn push_update(&self, id: &str, req: &Request) -> Response {
        super::push_upd::update(self, id, req).unwrap_or_else(|e| reply::error(e.status, &e.code))
    }
    fn push_delete(&self, id: &str, req: &Request) -> Response {
        super::push_upd::delete(self, id, req).unwrap_or_else(|e| reply::error(e.status, &e.code))
    }
}
