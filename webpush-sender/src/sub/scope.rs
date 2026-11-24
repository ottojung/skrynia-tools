use super::{CAP_LEN, Error, MAX_KEY, Result};

/// The sender only presents its scoped bearer; P1 verifies the durable grant.
pub fn check_scope(cap: &[u8], ns: &[u8], key: &[u8]) -> Result<()> {
    if cap.len() != CAP_LEN || ns.len() != CAP_LEN {
        return Err(Error::P1(9));
    }
    if key.is_empty() || key.len() > MAX_KEY {
        return Err(Error::Config("scope-key"));
    }
    if key.iter().any(|b| *b == 0 || *b < 0x20) {
        return Err(Error::Config("scope-bytes"));
    }
    Ok(())
}
