//! Negative cases against the real chain.
mod edge_auth;
mod p1_caps;
mod p1_caps_seed;
mod push;
mod push_cfg;
mod push_one;
mod push_seed;
pub fn run(art: &std::path::Path, tmp: &std::path::Path) -> Vec<String> {
    let mut ev = p1_caps::run(art, &tmp.join("caps"));
    ev.extend(edge_auth::run(art, &tmp.join("edge")));
    ev.extend(push::run(art, &tmp.join("push")));
    ev
}
