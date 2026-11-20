use super::{super::fixture, race};
use sha2::{Digest, Sha256};
#[test]
#[ignore = "requires real v2 guest, Rust Program0 generic lock and kernel PR #112"]
fn stale_sha_concurrent_replace_never_commits_twice() {
    let (s, token) = fixture::start(None);
    s.seed_file("v2/objects/team/race", b"original");
    let mut first = fixture::req("PUT", "team/race", &token, b"first");
    let mut second = fixture::req("PUT", "team/race", &token, b"second");
    let expected = hex::encode(Sha256::digest(b"original"));
    first
        .headers
        .insert("x-skrynia-if-sha256".into(), expected.clone());
    second
        .headers
        .insert("x-skrynia-if-sha256".into(), expected);
    let codes = race(&s, [first, second]);
    assert_eq!(codes.iter().filter(|&&s| s == 200).count(), 1);
    assert!(
        codes.iter().all(|s| matches!(s, 200 | 412 | 503)),
        "{codes:?}"
    );
    let result = s.read_file("v2/objects/team/race").unwrap();
    assert!(result == b"first" || result == b"second");
}
