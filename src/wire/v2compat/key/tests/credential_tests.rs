use super::credentials_api as credentials;
#[test]
fn credentials_have_exact_hex_sizes() {
    assert!(credentials::identifier(&"a".repeat(32)));
    assert!(credentials::sha256(&"a".repeat(64)));
    assert!(!credentials::identifier(&"a".repeat(31)));
    assert!(!credentials::sha256(&"A".repeat(64)));
}
