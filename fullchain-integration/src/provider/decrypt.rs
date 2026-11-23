//! RFC8291 aes128gcm decryption for disposable provider verification.
pub use super::decrypt_kdf::SubKeys;
use super::decrypt_kdf::derive;
use aes_gcm::{
    Aes128Gcm,
    aead::{Aead, KeyInit},
};
use p256::{PublicKey, SecretKey, ecdh, elliptic_curve::sec1::ToEncodedPoint};

pub fn decrypt(body: &[u8], sub: &SubKeys) -> Result<Vec<u8>, &'static str> {
    if body.len() < 16 + 4 + 1 + 65 + 16 {
        return Err("short");
    }
    let salt = &body[0..16];
    let idlen = body[20] as usize;
    if idlen != 65 || body.len() < 21 + idlen + 16 {
        return Err("idlen");
    }
    let keyid = &body[21..21 + idlen];
    let ct = &body[21 + idlen..];
    let client = SecretKey::from_slice(&sub.priv_bytes).map_err(|_| "subkey")?;
    let eph = PublicKey::from_sec1_bytes(keyid).map_err(|_| "eph")?;
    let shared = ecdh::diffie_hellman(client.to_nonzero_scalar(), eph.as_affine());
    let pub_pt = client.public_key().to_encoded_point(false);
    let (cek, nonce) = derive(
        shared.raw_secret_bytes(),
        &sub.auth,
        salt,
        pub_pt.as_bytes(),
        keyid,
    )?;
    let cipher = Aes128Gcm::new_from_slice(&cek).map_err(|_| "cipher")?;
    let n = aes_gcm::Nonce::from(nonce);
    let plain = cipher.decrypt(&n, ct).map_err(|_| "decrypt")?;
    let pad = plain.iter().rposition(|b| *b == 2).ok_or("pad")?;
    Ok(plain[..pad].to_vec())
}
