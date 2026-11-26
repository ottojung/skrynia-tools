use std::collections::HashMap;

#[derive(Default)]
pub struct Fake {
    pub input: Vec<u8>,
    pub verifier: [u8; 32],
    pub verifier_extra: Vec<u8>,
    pub fail_random: bool,
    pub files: HashMap<u32, Vec<u8>>,
    pub next_ticket: u32,
    pub collisions: u8,
    pub provider: i32,
    pub trap: bool,
    pub deny_delete: bool,
    pub fail_write: bool,
    pub fail_close: bool,
    pub invokes: Vec<(bool, u32)>,
    pub written: Vec<Vec<u8>>,
    pub deleted: Vec<Vec<u8>>,
    pub pos: usize,
    pub vpos: usize,
}

impl Fake {
    pub fn with_verifier(token: &[u8; 32]) -> Self {
        use sha2::{Digest, Sha256};
        Self {
            verifier: Sha256::digest(token).into(),
            ..Self::default()
        }
    }
}
