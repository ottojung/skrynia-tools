//! Deadline enforcement and source-level token-exposure guard.
use super::cases::{CAP, HOST, ORIGIN, edge};
use super::{HttpP1, MockEdge, P1Transport, Reply};

#[test]
fn deadline_and_source_guards() {
    let mock = MockEdge::start(Reply::Hang, HOST, ORIGIN, CAP, 1);
    let mut cfg = edge(mock.port, HOST, ORIGIN, CAP);
    cfg.timeout_ms = 100;
    assert!(HttpP1::new(cfg).unwrap().call(&[20]).is_err());
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/p1");
    let mut seen = Vec::new();
    scan(&dir, &mut seen);
    assert!(seen.is_empty(), "forbidden token symbols: {seen:?}");
}

fn scan(dir: &std::path::Path, seen: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            scan(&path, seen);
        } else if let Ok(text) = std::fs::read_to_string(&path) {
            let bad = [
                "operator".to_owned() + "_token",
                "OPERATOR".to_owned() + "_TOKEN",
                "X-Skrynia-".to_owned() + "Operator",
            ];
            if bad.iter().any(|needle| text.contains(needle)) {
                seen.push(path.display().to_string());
            }
        }
    }
}
