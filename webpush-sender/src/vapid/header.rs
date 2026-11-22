use super::VapidKey;
use base64ct::{Base64UrlUnpadded, Encoding};

/// Only the public verification key is exposed on the wire.
pub fn authorization(key: &VapidKey, jwt: &str) -> String {
    let public = Base64UrlUnpadded::encode_string(&key.public_bytes());
    format!("vapid t={jwt}, k={public}")
}
