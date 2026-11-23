//! Verifies VAPID and decrypts aes128gcm for a captured push request.
use super::decrypt::{SubKeys, decrypt};
use super::server::Seen;
use super::vapid;

pub struct Verified {
    pub plaintext: Vec<u8>,
    pub vapid_sub: String,
    pub vapid_k: String,
}

pub fn check(seen: &Seen, keys: &SubKeys, aud: &str) -> Result<Verified, String> {
    let authz = seen.header("authorization").ok_or("no authz")?;
    let v = vapid::verify(authz, aud).map_err(|e| e.to_string())?;
    let ct = seen.header("content-encoding").ok_or("no encoding")?;
    if ct != "aes128gcm" {
        return Err("encoding".into());
    }
    let plain = decrypt(&seen.body, keys).map_err(|e| e.to_string())?;
    Ok(Verified {
        plaintext: plain,
        vapid_sub: v.sub,
        vapid_k: v.k_b64,
    })
}
