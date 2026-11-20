use super::{CompatBackend, Request, Response, auth, reply as out, route_reply};
pub fn handle<B: CompatBackend>(b: &B, method: &str, path: &str, req: &Request) -> Response {
    if method == "POST" && path == "/grants" {
        return match auth::bearer(req) {
            Ok(t) => out::map(b.grant_create(t, &req.body), out::json),
            Err(e) => e,
        };
    }
    let Some(rest) = path.strip_prefix("/grants/") else {
        return route_reply::error(404, "not_found");
    };
    if let Some(id) = rest.strip_suffix("/users") {
        if method == "POST" && !id.is_empty() && !id.contains('/') {
            return match auth::issuer(req) {
                Ok(t) => out::map(b.grant_issue_user(id, t), out::json),
                Err(e) => e,
            };
        }
    }
    if method == "GET" && !rest.is_empty() && !rest.contains('/') {
        return match auth::bearer(req) {
            Ok(t) => out::map(b.grant_get(rest, t), out::json),
            Err(e) => e,
        };
    }
    if method == "DELETE" {
        if let Some((id, user)) = rest.split_once("/users/") {
            if !id.is_empty() && !user.is_empty() && !user.contains('/') {
                return match auth::issuer(req) {
                    Ok(t) => out::map(b.grant_revoke_user(id, user, t), |_| out::empty()),
                    Err(e) => e,
                };
            }
        } else if !rest.is_empty() && !rest.contains('/') {
            return match auth::issuer(req) {
                Ok(t) => out::map(b.grant_revoke(rest, t), |_| out::empty()),
                Err(e) => e,
            };
        }
    }
    route_reply::error(404, "not_found")
}
