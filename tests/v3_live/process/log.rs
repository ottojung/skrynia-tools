use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom},
    path::Path,
    process::Stdio,
};

fn path(dir: &Path) -> std::path::PathBuf {
    dir.join("server.stderr.log")
}

pub fn open(dir: &Path) -> Result<Stdio, String> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path(dir))
        .map_err(|e| e.to_string())?;
    Ok(Stdio::from(file))
}

pub fn tail(dir: &Path) -> String {
    let Ok(mut file) = File::open(path(dir)) else {
        return "log unavailable".into();
    };
    let Ok(size) = file.metadata().map(|m| m.len()) else {
        return "log metadata unavailable".into();
    };
    if file
        .seek(SeekFrom::Start(size.saturating_sub(4096)))
        .is_err()
    {
        return "log seek failed".into();
    }
    let mut bytes = Vec::new();
    if file.read_to_end(&mut bytes).is_err() {
        return "log read failed".into();
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

#[test]
fn bounded_diagnostic_tail() {
    use std::io::Write;
    let dir = std::env::temp_dir().join(format!("skrynia-v3-stderr-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let mut file = File::create(path(&dir)).unwrap();
    file.write_all(&vec![b'x'; 8192]).unwrap();
    file.write_all(b"root compilation timed out").unwrap();
    assert_eq!(tail(&dir).len(), 4096);
    assert!(tail(&dir).ends_with("root compilation timed out"));
    std::fs::remove_dir_all(&dir).unwrap();
}
