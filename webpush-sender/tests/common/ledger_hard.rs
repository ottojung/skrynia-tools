//! Ledger store hardening: bounded reads, private mode, stale temp safety.
use super::fixtures::config;
use webpush_sender::error::Error;
use webpush_sender::ledger::{FileLedgerStore, LedgerStore};

fn store_at(dir: &std::path::Path) -> (FileLedgerStore, std::path::PathBuf, std::path::PathBuf) {
    let cfg = config();
    let store = FileLedgerStore::from_config(dir, &cfg).unwrap();
    let b = webpush_sender::ledger::ledger_binding(&cfg);
    let hex: String = b.iter().map(|x| format!("{x:02x}")).collect();
    (
        store,
        dir.join(format!("ledger-{hex}.bin")),
        dir.join(format!("ledger-{hex}.tmp")),
    )
}

pub fn oversized_and_nonfile_rejected() {
    let dir = std::env::temp_dir().join(format!("sk-led-hard-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let (store, path, _) = store_at(&dir);
    std::fs::write(&path, vec![0u8; 20_000]).unwrap();
    assert_eq!(store.load().err(), Some(Error::Wire("ledger-shape")));
    std::fs::remove_file(&path).unwrap();
    std::fs::create_dir_all(&path).unwrap();
    assert_eq!(store.load().err(), Some(Error::Wire("ledger-shape")));
    let _ = std::fs::remove_dir_all(&dir);
}

#[cfg(unix)]
pub fn writes_private_mode() {
    use std::os::unix::fs::PermissionsExt;
    let dir = std::env::temp_dir().join(format!("sk-led-mode-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let (store, path, _) = store_at(&dir);
    store.save(b"probe").unwrap();
    let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
    let _ = std::fs::remove_dir_all(&dir);
}
#[cfg(not(unix))]
pub fn writes_private_mode() {}
