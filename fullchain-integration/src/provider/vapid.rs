//! Disposable VAPID verification for the local push provider.
use base64ct::{Base64UrlUnpadded, Encoding};
use p256::ecdsa::{Signature, VerifyingKey, signature::Verifier};

pub struct VapidInfo {
    pub aud: String,
    pub sub: String,
    pub exp: u64,
    pub k_b64: String,
}

pub fn verify(authz: &str, expected_aud: &str) -> Result<VapidInfo, &'static str> {
    let (t, k) = parse(authz)?;
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
    let exp = claims["exp"].as_u64().ok_or("exp")?;
    if aud != expected_aud {
        return Err("aud mismatch");
    }
    Ok(VapidInfo {
        aud,
        sub,
        exp,
        k_b64: k.to_owned(),
    })
}
fn parse(authz: &str) -> Result<(&str, &str), &'static str> {
    let a = authz
        .strip_prefix("vapid ")
        .or_else(|| authz.strip_prefix("Vapid "))
        .ok_or("scheme")?;
    let mut t = None;
    let mut k = None;
    for part in a.split(',') {
        let p = part.trim();
        if let Some(v) = p.strip_prefix("t=") {
            t = Some(v.trim());
        }
        if let Some(v) = p.strip_prefix("k=") {
            k = Some(v.trim());
        }
    }
    Ok((t.ok_or("t")?, k.ok_or("k")?))
}
