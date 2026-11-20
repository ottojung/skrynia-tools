use super::{super::fixture, race};
#[test]
#[ignore = "requires real v2 guest, Rust Program0 generic lock and kernel PR #112"]
fn two_concurrent_creates_never_both_succeed() {
    let (s, token) = fixture::start(None);
    let first = fixture::req("POST", "team/race", &token, b"first");
    let second = fixture::req("POST", "team/race", &token, b"second");
    let codes = race(&s, [first, second]);
    assert_eq!(codes.iter().filter(|&&s| s == 201).count(), 1);
    assert!(
        codes.iter().all(|s| matches!(s, 201 | 409 | 503)),
        "{codes:?}"
    );
    let final_bytes = s.read_file("v2/objects/team/race").unwrap();
    assert!(final_bytes == b"first" || final_bytes == b"second");
}
