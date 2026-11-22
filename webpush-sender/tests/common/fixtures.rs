use base64ct::{Base64UrlUnpadded, Encoding};
use p256::SecretKey;
use p256::elliptic_curve::sec1::ToEncodedPoint;
use webpush_sender::{Config, vapid::VapidKey};

pub const CAP: [u8; 32] = [7u8; 32];
pub const NS: [u8; 32] = [9u8; 32];
pub const PRIV: [u8; 32] = [3u8; 32];
pub const HINT: [u8; 32] = [0x5a; 32];
pub const KEY: &[u8] = b"notice/arrival";

pub fn config() -> Config {
    Config {
        sender_cap: CAP,
        ns: NS,
        key: KEY.to_vec(),
        max_payload: 4096,
        max_attempts: 3,
        base_backoff_ms: 10,
        max_backoff_ms: 100,
        ttl_secs: 60,
        topic: [2u8; 16],
        vapid_sub: "mailto:ops@example.invalid".to_owned(),
    }
}

pub fn vapid_key() -> VapidKey {
    VapidKey::from_slice(&PRIV).expect("valid disposable key")
}

pub fn sub_json(endpoint: &str) -> Vec<u8> {
    let secret = SecretKey::from_slice(&[11u8; 32]).expect("valid");
    let p256dh = secret.public_key().to_encoded_point(false);
    let auth = [42u8; 16];
    format!(
        r#"{{"endpoint":"{endpoint}","keys":{{"p256dh":"{}","auth":"{}"}}}}"#,
        Base64UrlUnpadded::encode_string(p256dh.as_bytes()),
        Base64UrlUnpadded::encode_string(&auth)
    )
    .into_bytes()
}

pub fn frame(subs: &[Vec<u8>]) -> Vec<u8> {
    let mut out = (subs.len() as u32).to_le_bytes().to_vec();
    for item in subs {
        out.extend_from_slice(&(item.len() as u32).to_le_bytes());
        out.extend_from_slice(item);
    }
    out
}
