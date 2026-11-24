use super::p1_caps_checks::negatives;
use super::p1_caps_support::{caps, setup};
use crate::{EdgeProc, Kernel, transport};

pub fn run(art: &std::path::Path, tmp: &std::path::Path) -> Vec<String> {
    let k = Kernel::start(art, &tmp.join("k")).expect("kernel");
    let e = EdgeProc::start(art, &tmp.join("e"), k.port, &k.token).expect("edge");
    let io = transport(&e);
    let c = caps();
    let key = b"notice/alpha";
    let sub = b"sub-a";
    let echo = std::fs::read(art.join("echo.wasm")).unwrap();
    setup(&io, &echo, &c, key, sub);
    negatives(&io, &c, key, sub)
}
