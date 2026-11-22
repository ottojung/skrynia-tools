mod common;

use base64ct::{Base64UrlUnpadded, Encoding};
use common::{PRIV, vapid_key};
use p256::ecdsa::{Signature, VerifyingKey, signature::Verifier};
use webpush_sender::vapid::authorization;

#[test]
fn jwt_is_real_es256_and_verifiable() {
    let key = vapid_key();
    let jwt = key
        .sign(
            "https://push.example:443",
            1_799_999_999,
            "mailto:ops@example.invalid",
        )
        .unwrap();
    let parts: Vec<&str> = jwt.split('.').collect();
    assert_eq!(parts.len(), 3);
    let header: serde_json::Value =
        serde_json::from_slice(&Base64UrlUnpadded::decode_vec(parts[0]).unwrap()).unwrap();
    assert_eq!(header["alg"], "ES256");
    assert_eq!(header["typ"], "JWT");
    let claims: serde_json::Value =
        serde_json::from_slice(&Base64UrlUnpadded::decode_vec(parts[1]).unwrap()).unwrap();
    assert_eq!(claims["aud"], "https://push.example:443");
    assert_eq!(claims["exp"], 1_799_999_999);
    let sig = Base64UrlUnpadded::decode_vec(parts[2]).unwrap();
    let verifier = VerifyingKey::from_sec1_bytes(&key.public_bytes()).unwrap();
    verifier
        .verify(
            format!("{}.{}", parts[0], parts[1]).as_bytes(),
            &Signature::from_slice(&sig).unwrap(),
        )
        .unwrap();
}

#[test]
fn authorization_exposes_only_public_key() {
    let key = vapid_key();
    let jwt = key
        .sign("https://push.example:443", 1, "mailto:x@example.invalid")
        .unwrap();
    let header = authorization(&key, &jwt);
    assert!(header.starts_with("vapid t="));
    assert!(header.contains(", k="));
    assert!(!header.as_bytes().windows(PRIV.len()).any(|w| w == PRIV));
    assert!(!jwt.as_bytes().windows(PRIV.len()).any(|w| w == PRIV));
}
