//! Host-side deterministic tests for target-ABI P0 (fake capabilities only).
mod arena;
mod descriptors;
mod frames;
mod hardening;
mod lock;
mod ops;
mod regressions;
mod seed;
mod support;
use super::{abi, alloc, auth, code, entry, fake, invoke, token};
use seed::seed;
use sha2::{Digest, Sha256};

#[test]
fn verifies_then_invokes_guest_exactly_once() {
    let bearer = [0x11u8; 32];
    let (i, o, _r) = seed(&bearer, &bearer, b"guest-bytes", b"opaque");
    assert_eq!(entry::skrynia_main(i, o), 0);
    fake::inspect(|b| {
        assert_eq!(b.eval_calls, 1);
        assert_eq!(b.eval_code, b"guest-bytes");
        assert_eq!(b.ram[0], alloc::FREE as u8);
    });
}

#[test]
fn wrong_bearer_is_denied_before_any_eval() {
    let (i, o, _r) = seed(&[0x22u8; 32], &[0x99u8; 32], b"guest", b"x");
    assert_eq!(entry::skrynia_main(i, o), auth::DENIED);
    fake::inspect(|s| assert_eq!(s.eval_calls, 0));
}

#[test]
fn verifier_is_a_digest_not_a_bearer() {
    let raw = [0x33u8; 32];
    let verifier: [u8; 32] = Sha256::digest(raw).into();
    assert!(token::matches(&raw, &verifier));
    assert!(!token::matches(&verifier, &verifier));
}
