use super::sync_dir;
use anyhow::Result;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

pub fn persist(root: &Path, name: &str, generation: u64) -> Result<()> {
    let directory = root.join(".locks");
    let destination = directory.join(format!("{name}.generation"));
    let temporary = directory.join(format!(
        ".{name}.generation-{:032x}",
        rand::random::<u128>()
    ));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    let mut file = options.open(&temporary)?;
    let result = (|| -> Result<()> {
        file.write_all(&generation.to_be_bytes())?;
        file.sync_all()?;
        fs::rename(&temporary, destination)?;
        sync_dir(&directory)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}
