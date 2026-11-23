//! RFC8291 HKDF key derivation for the disposable aes128gcm provider.
use hkdf::Hkdf;
use sha2::Sha256;

pub fn derive(
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
