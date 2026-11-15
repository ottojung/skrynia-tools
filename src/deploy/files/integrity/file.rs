use anyhow::{Result, bail};
use sha2::{Digest, Sha256};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::{fs::OpenOptions, io::Read, path::Path};

pub fn hash(path: &Path, size: u64, digest: &mut Sha256) -> Result<()> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    options.custom_flags(libc::O_NOFOLLOW);
    let mut file = options.open(path)?;
    let metadata = file.metadata()?;
    if !metadata.file_type().is_file() || metadata.len() != size {
        bail!("release file type or size changed during hashing");
    }
    let mut remaining = size;
    let mut buffer = [0u8; 32768];
    while remaining > 0 {
        let count = usize::try_from(remaining)
            .unwrap_or(usize::MAX)
            .min(buffer.len());
        let read = file.read(&mut buffer[..count])?;
        if read == 0 {
            bail!("release file truncated during hashing");
        }
        digest.update(&buffer[..read]);
        remaining -= read as u64;
    }
    if file.read(&mut buffer[..1])? != 0 {
        bail!("release file grew during hashing");
    }
    Ok(())
}
