//! Real kernel + operator P0 + P1 cross-namespace wake and outbound WebPush.
mod checks;
mod ctx;
mod grants;
mod pump;
mod setup;
mod stress;
mod wire;
use std::path::Path;

pub fn run(art: &Path, tmp: &Path) -> Vec<String> {
    let c = ctx::Ctx::new(art, tmp);
    setup::register(&c, art);
    grants::provision(&c);
    checks::verify(&c);
    pump::deliver(&c);
    stress::offline(&c);
    vec![
        "wake_crossns=two_writers".into(),
        "wake_stale_ack=preserves_generation".into(),
        "wake_provider_vapid_and_decrypt=ok".into(),
        "wake_offline_1040_writes=ok".into(),
        "wake_delete=advances_generation".into(),
    ]
}
