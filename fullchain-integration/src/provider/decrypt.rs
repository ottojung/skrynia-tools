//! RFC8291 aes128gcm decryption for disposable provider verification.
use aes_gcm::{
    Aes128Gcm,
    aead::{Aead, KeyInit},
};
use p256::{PublicKey, SecretKey, ecdh, elliptic_curve::sec1::ToEncodedPoint};

pub struct SubKeys {
    pub priv_bytes: [u8; 32],
    pub auth: [u8; 16],
}

pub fn decrypt(body: &[u8], sub: &SubKeys) -> Result<Vec<u8>, &'static str> {
    let (salt, keyid, ct) = frame::parse(body)?;
    let client = SecretKey::from_slice(&sub.priv_bytes).map_err(|_| "subkey")?;
    let eph = PublicKey::from_sec1_bytes(keyid).map_err(|_| "eph")?;
    let shared = ecdh::diffie_hellman(client.to_nonzero_scalar(), eph.as_affine());
    let pub_pt = client.public_key().to_encoded_point(false);
    let (cek, nonce) = super::decrypt_derive::derive(
        shared.raw_secret_bytes(),
        &sub.auth,
        salt,
        pub_pt.as_bytes(),
        keyid,
    )?;
    let cipher = Aes128Gcm::new_from_slice(&cek).map_err(|_| "cipher")?;
    let n = aes_gcm::Nonce::from(nonce);
    let plain = cipher.decrypt(&n, ct).map_err(|_| "decrypt")?;
    let pad = plain.iter().rposition(|b| *b != 0).ok_or("pad")?;
    if plain[pad] != 2 {
        return Err("pad");
    }
    Ok(plain[..pad].to_vec())
}

mod frame;
