//! Regression test for the 17:42Z denied-auth BUSY-slot leak.
use super::{alloc, auth, code, entry, fake};
use sha2::{Digest, Sha256};

fn bearer_frame(secret: &[u8; 32]) -> Vec<u8> {
    let mut f = vec![auth::VERSION as u8];
    f.extend_from_slice(secret);
    f.extend_from_slice(b"x");
    f
}

#[test]
fn denied_auth_never_leaks_a_busy_slot() {
    let bearer = [0x77u8; 32];
    let verifier: [u8; 32] = Sha256::digest(bearer).into();
    fake::seed(
        vec![],
        vec![
            (
                String::from_utf8_lossy(auth::VERIFIER_PATH).into_owned(),
                verifier.to_vec(),
            ),
            (
                String::from_utf8_lossy(code::GUEST_PATH).into_owned(),
                b"guest".to_vec(),
            ),
        ],
    );
    for _ in 0..(alloc::SLOTS + 2) {
        let (i, o) = fake::request(bearer_frame(&[0x22; 32]));
        assert_eq!(entry::skrynia_main(i, o), auth::DENIED);
    }
    let (i, o) = fake::request(bearer_frame(&bearer));
    assert_eq!(entry::skrynia_main(i, o), 0);
    fake::inspect(|s| {
        let meta = &s.ram[alloc::META_AT as usize..(alloc::META_AT + alloc::SLOTS) as usize];
        assert!(meta.iter().all(|&b| b == alloc::FREE as u8));
    });
}
