//! Negative cases against the real chain.
mod edge_auth;
mod p1_caps;
mod p1_caps_checks;
mod p1_caps_report;
mod p1_caps_support;
mod push;
mod push_one;
mod push_pump;
mod push_setup;
pub fn run(art: &std::path::Path, tmp: &std::path::Path) -> Vec<String> {
    let mut ev = p1_caps::run(art, &tmp.join("caps"));
    ev.extend(edge_auth::run(art, &tmp.join("edge")));
    ev.extend(push::run(art, &tmp.join("push")));
    ev
}
