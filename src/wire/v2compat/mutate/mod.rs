use super::read::cache;
use super::{
    backend::{CompatBackend, CompatResult, Mutation, MutationKind, ObjectRead},
    key, reply,
    types::{Request, Response},
};
mod precondition;
mod response;
mod token;

pub fn handle<B: CompatBackend>(backend: &B, method: &str, raw: &str, req: &Request) -> Response {
    let key = match key::decode(raw) {
        Ok(value) => value,
        Err(e) => return reply::error(400, e),
    };
    let expected = match precondition::check(req, method) {
        Ok(v) => v,
        Err(e) => return e,
    };
    if method == "DELETE" {
        match backend.read_object(&key) {
            Ok(ObjectRead::Missing) => return reply::error(404, "not_found"),
            Err(e) => return reply::error(e.status, &e.code),
            _ => {}
        }
    }
    let token = match token::get(req) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let kind = match method {
        "POST" => MutationKind::Create,
        "PUT" => MutationKind::Replace,
        _ => MutationKind::Delete,
    };
    let input = Mutation {
        key: &key,
        kind,
        token,
        expected: expected.as_deref(),
        body: if method == "DELETE" { &[] } else { &req.body },
    };
    super::mutate::response::finish(backend.mutate_object(input), method)
}
