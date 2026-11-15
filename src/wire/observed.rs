use super::decode;
use anyhow::{Result, bail};

/// Whether existing immutable log bytes prove the operation has committed.
pub fn observed(previous: &[u8], wanted: &[u8], operation_id: &str) -> Result<bool> {
    if let Ok((existing_id, _)) = decode(previous)
        && existing_id == operation_id
    {
        if previous == wanted {
            return Ok(true);
        }
        bail!("operation ID reused with different payload");
    }
    Ok(false)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detects_exact_replay_and_divergent_payload() {
        let one = super::super::encode("00000000000000000000000000000001", b"one").unwrap();
        let two = super::super::encode("00000000000000000000000000000001", b"two").unwrap();
        assert!(observed(&one, &one, "00000000000000000000000000000001").unwrap());
        assert!(observed(&one, &two, "00000000000000000000000000000001").is_err());
    }
}
