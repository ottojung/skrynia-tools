//! RFC8291 aes128gcm decryption for disposable provider verification.
use aes_gcm::{
    Aes128Gcm,
    aead::{Aead, KeyInit},
};
use hkdf::Hkdf;
use p256::{PublicKey, SecretKey, ecdh, elliptic_curve::sec1::ToEncodedPoint};
use sha2::Sha256;

pub struct SubKeys {
    pub priv_bytes: [u8; 32],
    pub auth: [u8; 16],
}

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
fn derive(
    shared: &[u8],
    auth: &[u8],
    salt: &[u8],
    client: &[u8],
    server: &[u8],
) -> Result<([u8; 16], [u8; 12]), &'static str> {
    let (_, info_hk) = Hkdf::<Sha256>::extract(Some(auth), shared);
    let mut info = b"WebPush: info\0".to_vec();
    info.extend_from_slice(client);
    info.extend_from_slice(server);
    let mut ikm = [0u8; 32];
    info_hk.expand(&info, &mut ikm).map_err(|_| "info")?;
    let (_, ckey) = Hkdf::<Sha256>::extract(Some(salt), &ikm);
    let mut cek = [0u8; 16];
    let mut nonce = [0u8; 12];
    ckey.expand(b"Content-Encoding: aes128gcm\0", &mut cek)
        .map_err(|_| "cek")?;
    ckey.expand(b"Content-Encoding: nonce\0", &mut nonce)
        .map_err(|_| "nonce")?;
    Ok((cek, nonce))
}
