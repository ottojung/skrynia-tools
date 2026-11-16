use anyhow::{Result, bail};
use sha2::{Digest, Sha256};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::{fs::OpenOptions, io::Read, path::Path};

pub fn sha256(path: &Path, expected_size: u64) -> Result<String> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    options.custom_flags(libc::O_NOFOLLOW);
    let mut file = options.open(path)?;
    if !file.metadata()?.is_file() {
        bail!("artifact is not a regular file");
    }
    let mut digest = Sha256::new();
    let mut bytes = 0u64;
    let mut buffer = [0u8; 32768];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        bytes = bytes
            .checked_add(count as u64)
            .ok_or_else(|| anyhow::anyhow!("artifact size overflow"))?;
        if bytes > expected_size {
            bail!("artifact grew during diff");
        }
        digest.update(&buffer[..count]);
    }
    if bytes != expected_size {
        bail!("artifact truncated during diff");
    }
    Ok(hex::encode(digest.finalize()))
}
