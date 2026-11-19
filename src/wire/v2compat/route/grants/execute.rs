use super::{Action, Backend, Req, Resp, preply};
fn execute<B: Backend>(
    backend: &B,
    action: Action<'_>,
    token: &str,
    req: &Req,
) -> Result<serde_json::Value, Resp> {
    let result = match action {
        Action::Create => {
            if req.body.len() > 8192 {
                return Err(preply::error(400, "request_too_large"));
            }
            let spec = serde_json::from_slice(&req.body)
                .map_err(|_| preply::error(400, "invalid_json"))?;
            backend.grant_create(token, &spec)
        }
        Action::Inspect(id) => backend.grant_inspect(id, token),
        Action::Issue(id) => backend.grant_issue(id, token),
        Action::RevokeUser(id, user) => backend.grant_revoke_user(id, user, token),
        Action::Revoke(id) => backend.grant_revoke(id, token),
    };
    result.map_err(|e| preply::error(e.status, &e.code))
}
pub fn run<B: Backend>(backend: &B, action: Action<'_>, token: &str, req: &Req) -> Resp {
    match execute(backend, action, token, req) {
        Ok(value) => match serde_json::to_vec(&value) {
            Ok(body) => preply::json(200, body),
            Err(_) => preply::error(500, "invalid_state"),
        },
        Err(reply) => reply,
    }
}
