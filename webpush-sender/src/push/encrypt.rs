//! RFC 8291 aes128gcm record; all randomness is injected.
use super::{Error, Result, Subscription, keys};
use aes_gcm::{
    Aes128Gcm,
    aead::{Aead, KeyInit},
};
use p256::{PublicKey, SecretKey, ecdh, elliptic_curve::sec1::ToEncodedPoint};

pub trait Rng {
    fn fill(&self, out: &mut [u8]);
}

pub(super) const MAX_BODY: usize = 4096;
const HEADER: usize = 86;
const TAG: usize = 16;
const MAX_PLAINTEXT: usize = MAX_BODY - HEADER - TAG - 1;

pub fn encrypt(payload: &[u8], sub: &Subscription, rng: &impl Rng) -> Result<Vec<u8>> {
    if payload.len() > MAX_PLAINTEXT {
        return Err(Error::Quota);
    }
    let mut salt = [0u8; 16];
    let mut raw = [0u8; 32];
    rng.fill(&mut salt);
    rng.fill(&mut raw);
    let eph = SecretKey::from_slice(&raw).map_err(|_| Error::Crypto("eph"))?;
    let client = PublicKey::from_sec1_bytes(&sub.p256dh).map_err(|_| Error::Crypto("p256dh"))?;
    let shared = ecdh::diffie_hellman(eph.to_nonzero_scalar(), client.as_affine());
    let server = eph.public_key().to_encoded_point(false);
    let rk = keys::derive(
        shared.raw_secret_bytes(),
        &sub.auth,
        &salt,
        &sub.p256dh,
        server.as_bytes(),
    )?;
    let cipher = Aes128Gcm::new_from_slice(&rk.cek).map_err(|_| Error::Crypto("cipher"))?;
    let nonce = aes_gcm::Nonce::from(rk.nonce);
    let mut plain = payload.to_vec();
    plain.push(2);
    let sealed = cipher
        .encrypt(&nonce, plain.as_slice())
        .map_err(|_| Error::Crypto("seal"))?;
    let mut out = Vec::with_capacity(HEADER + sealed.len());
    out.extend_from_slice(&salt);
    out.extend_from_slice(&(MAX_BODY as u32).to_be_bytes());
    out.push(65);
    out.extend_from_slice(server.as_bytes());
    out.extend_from_slice(&sealed);
    Ok(out)
}

#[cfg(test)]
mod tests;
