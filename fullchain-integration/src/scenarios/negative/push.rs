//! Redirect and bounded-retry negatives for the real full chain.
use super::push_one::one;
use crate::provider::ReplyPlan;

pub fn run(art: &std::path::Path, tmp: &std::path::Path) -> Vec<String> {
    let (sub_json, _) = crate::scenarios::keys::fixture();
    let mut plan302 = ReplyPlan::status(302);
    plan302
        .headers
        .push(("Location".into(), "https://evil.test/x".into()));
    let p302 = one(art, &tmp.join("r302"), &sub_json, plan302);
    let p503 = one(
        art,
        &tmp.join("r503"),
        &sub_json,
        ReplyPlan::status(503).retry(1),
    );
    vec![
        format!("neg_redirect_err={p302}"),
        format!("neg_backoff_attempts={p503}"),
    ]
}
