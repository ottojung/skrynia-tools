mod fake;
mod freshness;
mod ranges;
use super::{
    CompatBackend, CompatResult, Meta, Object, ObjectRead, Push, Request, Response, Store,
    Unavailable,
};
use fake::{fake, request};
fn handle(backend: &impl CompatBackend, req: &Request) -> Response {
    super::handle(
        backend,
        &req.method,
        req.path.trim_start_matches("/store/"),
        req,
    )
}
