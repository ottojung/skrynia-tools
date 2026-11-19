use super::{CompatBackend, Request, Response, reply};

pub fn handle<B: CompatBackend>(backend: &B, req: &Request) -> Response {
    let p = super::list_query::params(req.query.as_deref());
    let Some(locations) = p.get("locations") else {
        return reply::error(400, "locations_required");
    };
    let limit = p
        .get("limit")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(1000);
    let token = req
        .headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("authorization"))
        .and_then(|(_, v)| v.strip_prefix("Bearer "))
        .unwrap_or("");
    match backend.list_objects(token, locations, p.get("after").map(String::as_str), limit) {
        Ok(page) => {
            let mut body = serde_json::json!({ "keys": page.keys });
            if let Some(next) = page.next {
                body["next"] = serde_json::Value::String(next);
            }
            reply::json(200, serde_json::to_vec(&body).unwrap_or_default())
        }
        Err(e) => reply::error(e.status, &e.code),
    }
}
