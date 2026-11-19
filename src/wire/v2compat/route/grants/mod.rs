mod action;
mod auth;
mod execute;
use super::{Backend, Req, Resp, preply};
pub(super) use action::Action;
pub fn handle<B: Backend>(backend: &B, method: &str, path: &str, req: &Req) -> Option<Resp> {
    let action = Action::parse(method, path)?;
    let token = match auth::token(req, action.issuer_only()) {
        Ok(token) => token,
        Err(response) => return Some(response),
    };
    Some(execute::run(backend, action, token, req))
}
