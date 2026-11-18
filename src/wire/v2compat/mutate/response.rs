use super::CompatResult;
use super::{Response, reply};
pub fn finish(result: CompatResult<bool>, method: &str) -> Response {
    match result {
        Ok(pending) => {
            let body = if pending {
                br#"{"ok":true,"notificationPending":true}"#.to_vec()
            } else {
                br#"{"ok":true}"#.to_vec()
            };
            reply::json(if method == "POST" { 201 } else { 200 }, body)
        }
        Err(e) => reply::error(e.status, &e.code),
    }
}
