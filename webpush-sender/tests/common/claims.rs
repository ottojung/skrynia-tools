//! Decode the `aud` claim from an injected VAPID Authorization header.
use base64ct::{Base64UrlUnpadded, Encoding};

pub fn jwt_aud(header: &str) -> String {
    let jwt = header
        .split_once("vapid t=")
        .unwrap()
        .1
        .split(", k=")
        .next()
        .unwrap();
    let raw = Base64UrlUnpadded::decode_vec(jwt.split('.').nth(1).unwrap()).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    value["aud"].as_str().unwrap().to_owned()
}

/// Decode all claims from a compact JWT (no Authorization wrapper).
pub fn jwt_claims(jwt: &str) -> serde_json::Value {
    let raw = Base64UrlUnpadded::decode_vec(jwt.split('.').nth(1).unwrap()).unwrap();
    serde_json::from_slice(&raw).unwrap()
}
