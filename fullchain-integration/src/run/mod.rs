//! Harness entrypoint: runs real full-chain scenarios.
use std::path::PathBuf;
mod tmp;

pub fn execute() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let art = std::env::var("FC_ARTIFACTS")
        .map(PathBuf::from)
        .unwrap_or_else(|_| root.join("../artifacts"));
    let requested = std::env::var_os("FC_TMP").map(PathBuf::from);
    let tmp = tmp::create(requested).expect("FC_TMP must name a new private directory");
    eprintln!("artifacts={}", art.display());
    eprintln!("tmp={}", tmp.display());
    if let Err(e) = crate::chain::verify_all(&art) {
        panic!("artifact SHA256 pin check FAILED (fail closed): {e}");
    }
    println!("pin_verify=ok");
    for line in crate::chain::manifest(&art).expect("runtime manifest") {
        println!("{line}");
    }
    let mut ev = crate::scenarios::run_positive(&art, &tmp.join("pos"));
    ev.extend(crate::scenarios::run_negative(&art, &tmp.join("neg")));
    ev.extend(crate::scenarios::run_wake(&art, &tmp.join("wake")));
    for e in &ev {
        crate::evidence::line("EVIDENCE", e);
    }
    println!("FULLCHAIN_OK");
}
