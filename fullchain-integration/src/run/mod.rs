//! Harness entrypoint: runs real full-chain scenarios.
use std::path::PathBuf;

pub fn execute() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let art = std::env::var("FC_ARTIFACTS")
        .map(PathBuf::from)
        .unwrap_or_else(|_| root.join("../artifacts"));
    let tmp = std::env::var("FC_TMP")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir().join("fc-webpush-20261123"));
    if tmp.exists() {
        std::fs::remove_dir_all(&tmp).ok();
    }
    std::fs::create_dir_all(&tmp).expect("tmp");
    eprintln!("artifacts={}", art.display());
    eprintln!("tmp={}", tmp.display());
    let mut ev = crate::scenarios::run_positive(&art, &tmp.join("pos"));
    ev.extend(crate::scenarios::run_negative(&art, &tmp.join("neg")));
    for e in &ev {
        println!("EVIDENCE {e}");
    }
    println!("FULLCHAIN_OK");
}
