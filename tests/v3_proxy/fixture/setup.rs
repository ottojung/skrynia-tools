use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};
pub fn prepare(guest: Option<&[u8]>) -> PathBuf {
    let temp = std::env::temp_dir().join(format!(
        "skrynia-dumb-proxy-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    ));
    fs::create_dir_all(temp.join("programs")).unwrap();
    fs::create_dir_all(temp.join("security")).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(temp.join("security"), fs::Permissions::from_mode(0o700)).unwrap();
    }
    let digest = Sha256::digest([0x42u8; 32]);
    let path = temp.join("security/program0.sha256");
    fs::write(&path, digest).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
    }
    if let Some(code) = guest {
        fs::write(temp.join("programs/root.wasm"), code).unwrap();
    }
    temp
}
