//! Ordinary subscribed key with full legacy queue and coalesced fallback.
mod auth;
mod ctx;
mod pump;
mod tests;
mod wire;
use std::path::Path;
pub fn run(art: &Path, tmp: &Path) -> Vec<String> {
    let c = ctx::Ctx::new(art, tmp);
    auth::setup(&c, art);
    tests::fill_and_check(&c);
    pump::deliver(&c);
    vec![
        "ordinary_hint_budget=256".into(),
        "ordinary_fallback_highbit=real_wasm".into(),
        "ordinary_fallback_vapid_decrypt=ok".into(),
        "ordinary_fallback_sender_ack=ok".into(),
    ]
}
