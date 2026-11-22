//! Crash-safe replace: write temp, fsync file, rename, fsync parent directory.
//! A failure at any step leaves the previous committed file intact.
use super::{Error, Result};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

pub fn replace(final_path: &Path, tmp_path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(tmp_path)
        .map_err(|e| classify(e, "ledger-open"))?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| classify(e, "ledger-sync"))?;
    drop(file);
    std::fs::rename(tmp_path, final_path).map_err(|_| Error::Transport("ledger-rename"))?;
    let dir =
        File::open(final_path.parent().unwrap()).map_err(|_| Error::Transport("ledger-dir"))?;
    dir.sync_all().map_err(|e| classify(e, "ledger-dir-sync"))
}

fn classify(e: std::io::Error, label: &'static str) -> Error {
    if e.kind() == std::io::ErrorKind::StorageFull || e.raw_os_error() == Some(28) {
        Error::Quota
    } else {
        Error::Transport(label)
    }
}

#[cfg(test)]
mod test {
    use super::replace;
    use std::fs;

    #[test]
    fn atomic_replace_commits_and_cleans_temp() {
        let dir = std::env::temp_dir().join(format!("skrynia-ledger-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let final_path = dir.join("ledger.bin");
        let tmp_path = dir.join("ledger.tmp");
        replace(&final_path, &tmp_path, b"payload").unwrap();
        assert_eq!(fs::read(&final_path).unwrap(), b"payload");
        assert!(!tmp_path.exists());
        fs::remove_dir_all(&dir).unwrap();
    }
}
