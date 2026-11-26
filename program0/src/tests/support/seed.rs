//! Authenticated request fixture shared by host-side tests.
use super::fake;
use crate::{auth, code};
use sha2::{Digest, Sha256};

pub fn seed(bearer: &[u8; 32], sent: &[u8; 32], guest: &[u8], payload: &[u8]) -> (i32, i32, i32) {
    let frame = [&[1u8][..], sent, payload].concat();
    let verifier: [u8; 32] = Sha256::digest(bearer).into();
    fake::seed(
        frame,
        vec![
            (
                String::from_utf8_lossy(auth::VERIFIER_PATH).into_owned(),
                verifier.to_vec(),
            ),
            (
                String::from_utf8_lossy(code::GUEST_PATH).into_owned(),
                guest.to_vec(),
            ),
        ],
    )
}
