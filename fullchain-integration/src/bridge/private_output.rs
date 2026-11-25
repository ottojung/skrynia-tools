use std::fs::{File, OpenOptions};
use std::io;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

pub fn reserve(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
}

#[cfg(test)]
mod tests {
    use super::reserve;
    use std::{fs, io::Write, os::unix::fs::PermissionsExt};

    #[test]
    fn reserve_refuses_existing_file_and_keeps_new_output_private() {
        let dir = std::env::temp_dir().join(format!("bridge-reserve-{}", std::process::id()));
        fs::create_dir(&dir).unwrap();
        let path = dir.join("hint");
        let mut out = reserve(&path).unwrap();
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        out.write_all(b"fixture").unwrap();
        assert!(reserve(&path).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"fixture");
        drop(out);
        fs::remove_dir_all(dir).unwrap();
    }
}
