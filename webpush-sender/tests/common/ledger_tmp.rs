//! Stale temp and symlink safety for ledger replace.
use super::fixtures::config;
use webpush_sender::ledger::{FileLedgerStore, LedgerStore};

pub fn stale_tmp_and_symlink_safe() {
    let dir = std::env::temp_dir().join(format!("sk-led-tmp-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let cfg = config();
    let store = FileLedgerStore::from_config(&dir, &cfg).unwrap();
    let b = webpush_sender::ledger::ledger_binding(&cfg);
    let hex: String = b.iter().map(|x| format!("{x:02x}")).collect();
    let tmp = dir.join(format!("ledger-{hex}.tmp"));
    std::fs::write(&tmp, b"stale").unwrap();
    store.save(b"ok").unwrap();
    #[cfg(unix)]
    {
        let _ = std::fs::remove_file(&tmp);
        std::os::unix::fs::symlink("/etc/passwd", &tmp).unwrap();
        store.save(b"ok2").unwrap();
        assert!(!tmp.exists());
    }
    std::fs::remove_dir_all(&dir).unwrap();
}
