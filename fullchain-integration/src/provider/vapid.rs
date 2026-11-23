//! Disposable VAPID verification for the local push provider.
use base64ct::{Base64UrlUnpadded, Encoding};
use p256::ecdsa::{Signature, VerifyingKey, signature::Verifier};

pub struct VapidInfo {
    pub sub: String,
    pub k_b64: String,
}

pub fn verify(authz: &str, expected_aud: &str) -> Result<VapidInfo, &'static str> {
    let (t, k) = super::vapid_parse::parse(authz)?;
    let parts: Vec<&str> = t.split('.').collect();
    if parts.len() != 3 {
        return Err("jwt parts");
    }
    let sig = Base64UrlUnpadded::decode_vec(parts[2]).map_err(|_| "sig b64")?;
    let sig = Signature::from_slice(&sig).map_err(|_| "sig")?;
    let pk = Base64UrlUnpadded::decode_vec(k).map_err(|_| "k b64")?;
    let vk = VerifyingKey::from_sec1_bytes(&pk).map_err(|_| "vk")?;
    let msg = format!("{}.{}", parts[0], parts[1]);
    vk.verify(msg.as_bytes(), &sig).map_err(|_| "verify")?;
    let claims: serde_json::Value =
        serde_json::from_slice(&Base64UrlUnpadded::decode_vec(parts[1]).map_err(|_| "claims b64")?)
            .map_err(|_| "claims json")?;
    let aud = claims["aud"].as_str().ok_or("aud")?.to_owned();
    let sub = claims["sub"].as_str().ok_or("sub")?.to_owned();
    let _exp = claims["exp"].as_u64().ok_or("exp")?;
    if aud != expected_aud {
        return Err("aud mismatch");
    }
    Ok(VapidInfo {
        sub,
        k_b64: k.to_owned(),
    })
}
