use sha2::{Digest, Sha256};

pub const CAP_LEN: usize = 32;

pub fn read_cap(write_cap: &[u8]) -> [u8; CAP_LEN] {
    let mut h = Sha256::new();
    h.update(write_cap);
    h.finalize().into()
}

pub fn read_cap_vec(write_cap: &[u8]) -> Vec<u8> {
    read_cap(write_cap).to_vec()
}

pub fn hash_bytes(bytes: &[u8]) -> [u8; CAP_LEN] {
    let mut h = Sha256::new();
    h.update(bytes);
    h.finalize().into()
}

pub fn valid_caps(write_cap: &[u8], read_cap_claim: &[u8]) -> bool {
    write_cap.len() == CAP_LEN
        && read_cap_claim.len() == CAP_LEN
        && read_cap(write_cap)[..] == read_cap_claim[..]
}
