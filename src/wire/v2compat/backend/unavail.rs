use super::push::Push;
use super::{Request, Response, Unavailable, reply};
impl Push for Unavailable {
    fn push_subscribe(&self, _req: &Request) -> Response {
        reply::error(501, "not_implemented")
    }
    fn push_update(&self, _id: &str, _req: &Request) -> Response {
        reply::error(501, "not_implemented")
    }
    fn push_delete(&self, _id: &str, _req: &Request) -> Response {
        reply::error(501, "not_implemented")
    }
}
