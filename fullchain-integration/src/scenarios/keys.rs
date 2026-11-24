//! Deterministic subscription key material for disposable verification.
use crate::SubKeys;
use base64ct::{Base64UrlUnpadded, Encoding};
use p256::elliptic_curve::sec1::ToEncodedPoint;

pub fn fixture() -> (Vec<u8>, SubKeys) {
    let priv_bytes = [11u8; 32];
    let auth = [42u8; 16];
    let secret = p256::SecretKey::from_slice(&priv_bytes).expect("sk");
    let p256dh = secret.public_key().to_encoded_point(false);
    let json = serde_json::json!({
        "endpoint": "https://push.example.test/push/sub",
        "keys": {
            "p256dh": Base64UrlUnpadded::encode_string(p256dh.as_bytes()),
            "auth": Base64UrlUnpadded::encode_string(&auth),
        }
    })
    .to_string()
    .into_bytes();
    (json, SubKeys { priv_bytes, auth })
}
