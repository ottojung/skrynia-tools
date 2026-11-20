use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

/// Authenticate the raw bearer secret, not the public verifier digest.
pub fn matches(raw: &[u8; 32], expected_sha256: &[u8; 32]) -> bool {
    let digest: [u8; 32] = Sha256::digest(raw).into();
    bool::from(digest.ct_eq(expected_sha256))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_only_the_original_256_bit_secret() {
        let secret = [0x42u8; 32];
        let verifier: [u8; 32] = Sha256::digest(secret).into();
        assert!(matches(&secret, &verifier));
        assert!(!matches(&verifier, &verifier));
        for pos in 0..32 {
            let mut tampered = secret;
            tampered[pos] ^= 1;
            assert!(!matches(&tampered, &verifier));
        }
    }
}
