use super::{Backend, Req, Resp};
pub fn handle<B: Backend>(backend: &B, method: &str, path: &str, req: &Req) -> Option<Resp> {
    Some(match (method, path) {
        ("POST", "/push/subscriptions") => backend.push_subscribe(req),
        ("PUT", p) if p.starts_with("/push/subscriptions/") => backend.push_update(&p[20..], req),
        ("DELETE", p) if p.starts_with("/push/subscriptions/") => {
            backend.push_delete(&p[20..], req)
        }
        _ => return None,
    })
}
