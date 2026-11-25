//! Ledger store hardening: bounded reads and private durable writes.
use super::fixtures::config;
use webpush_sender::error::Error;
use webpush_sender::ledger::{FileLedgerStore, LedgerStore};

pub fn oversized_and_nonfile_rejected() {
    let dir = std::env::temp_dir().join(format!("sk-led-hard-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let cfg = config();
    let store = FileLedgerStore::from_config(&dir, &cfg).unwrap();
    let name = std::fs::read_dir(&dir)
        .map(|_| "unused")
        .ok();
    let _ = name;
    // Write an oversized file at the exact ledger path.
    let binding = webpush_sender::ledger::ledger_binding(&cfg);
    let hex: String = binding.iter().map(|b| format!("{b:02x}")).collect();
    let path = dir.join(format!("ledger-{hex}.bin"));
    std::fs::write(&path, vec![0u8; 20_000]).unwrap();
    assert_eq!(store.load().err(), Some(Error::Wire("ledger-shape")));
    std::fs::remove_file(&path).unwrap();
    std::fs::create_dir_all(&path).unwrap();
    assert_eq!(store.load().err(), Some(Error::Wire("ledger-shape")));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[cfg(unix)]
pub fn writes_private_mode() {
    use std::os::unix::fs::PermissionsExt;
    let dir = std::env::temp_dir().join(format!("sk-led-mode-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let cfg = config();
    let store = FileLedgerStore::from_config(&dir, &cfg).unwrap();
    store.save(b"probe").unwrap();
    let binding = webpush_sender::ledger::ledger_binding(&cfg);
    let hex: String = binding.iter().map(|b| format!("{b:02x}")).collect();
    let path = dir.join(format!("ledger-{hex}.bin"));
    let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
    std::fs::remove_dir_all(&dir).unwrap();
}
#[cfg(not(unix))]
pub fn writes_private_mode() {}
