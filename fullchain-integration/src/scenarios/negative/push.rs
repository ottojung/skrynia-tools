use super::push_one::one;
use crate::ReplyPlan;
use webpush_sender::Error;

pub fn run(art: &std::path::Path, tmp: &std::path::Path) -> Vec<String> {
    let (sub_json, _) = crate::fixture();
    let mut plan302 = ReplyPlan::status(302);
    plan302
        .headers
        .push(("Location".into(), "https://evil.test/x".into()));
    let (err302, seen302) = one(art, &tmp.join("r302"), &sub_json, plan302);
    assert_eq!(
        err302,
        Error::Transport("status"),
        "302 redirect is rejected"
    );
    assert_eq!(seen302, 1, "redirect must not be followed");
    let (err503, seen503) = one(
        art,
        &tmp.join("r503"),
        &sub_json,
        ReplyPlan::status(503).retry(1),
    );
    assert_eq!(err503, Error::Exhausted, "503 retry budget exhausted");
    assert_eq!(seen503, 3, "retry capped at three sends");
    vec![
        format!("neg_redirect_err={err302:?}/seen={seen302}"),
        format!("neg_backoff_attempts={err503:?}/seen={seen503}"),
    ]
}
