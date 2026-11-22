//! Real VAPID-format signing: ES256 (P-256) over a compact JWT.
use super::{Error, Result};
use base64ct::{Base64UrlUnpadded, Encoding};
use p256::ecdsa::{SigningKey, signature::Signer};

pub struct VapidKey(SigningKey);

impl VapidKey {
    pub fn from_slice(bytes: &[u8]) -> Result<Self> {
        let key = SigningKey::from_slice(bytes).map_err(|_| Error::Crypto("vapid-key"))?;
        Ok(Self(key))
    }

    pub fn public_bytes(&self) -> Vec<u8> {
        self.0
            .verifying_key()
            .to_encoded_point(false)
            .as_bytes()
            .to_vec()
    }

    pub fn sign(&self, aud: &str, exp: u64, sub: &str) -> Result<String> {
        let header = Base64UrlUnpadded::encode_string(br#"{"alg":"ES256","typ":"JWT"}"#);
        let payload = format!(r#"{{"aud":"{aud}","exp":{exp},"sub":"{sub}"}}"#);
        let claims = Base64UrlUnpadded::encode_string(payload.as_bytes());
        let input = format!("{header}.{claims}");
        let sig: p256::ecdsa::Signature = self.0.sign(input.as_bytes());
        let tail = Base64UrlUnpadded::encode_string(&sig.to_bytes());
        Ok(format!("{input}.{tail}"))
    }
}
