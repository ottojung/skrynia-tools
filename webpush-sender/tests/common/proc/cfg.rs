//! Hostile config files, bounded before allocation. No external I/O.
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use webpush_sender::push::proc::RuntimeConfig;

fn body(dir: &Path) -> PathBuf {
    let p = dir.join("cfg.json");
    let text = format!(
        r#"{{"edge":{{"ip":"127.0.0.1","port":8443,"host":"h","origin":"o","capability":"capability-012345","timeout_ms":5000,"max_request_bytes":4096,"max_response_bytes":4096}},"sender_cap":"{c}","ns":"{c}","key":"notice/arrive","max_payload":4096,"max_attempts":3,"base_backoff_ms":10,"max_backoff_ms":100,"ttl_secs":60,"topic":"{t}","vapid_sub":"mailto:ops@example.invalid","ledger_epoch":0,"vapid_private_key":"{c}","ledger_dir":"{d}","poll_ms":1000}}"#,
        c = "11".repeat(32),
        t = "22".repeat(16),
        d = dir.join("ledgers").display()
    );
    fs::write(&p, text).unwrap();
    fs::set_permissions(&p, fs::Permissions::from_mode(0o600)).unwrap();
    p
}

pub fn load_and_hostile() {
    let dir = std::env::temp_dir().join(format!("rt-cfg-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("ledgers")).unwrap();
    let p = body(&dir);
    assert_eq!(RuntimeConfig::from_path(&p).unwrap().poll_ms, 1000);
    fs::set_permissions(&p, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(RuntimeConfig::from_path(&p).is_err());
    fs::set_permissions(&p, fs::Permissions::from_mode(0o600)).unwrap();
    let big = dir.join("big");
    fs::File::create(&big).unwrap().set_len(1 << 30).unwrap();
    assert!(RuntimeConfig::from_path(&big).is_err());
    let fifo = dir.join("fifo");
    assert!(
        std::process::Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .unwrap()
            .success()
    );
    assert!(RuntimeConfig::from_path(&fifo).is_err());
    let link = dir.join("link");
    std::os::unix::fs::symlink(&p, &link).unwrap();
    assert!(RuntimeConfig::from_path(&link).is_err());
    let hard = dir.join("hard");
    fs::hard_link(&p, &hard).unwrap();
    assert!(RuntimeConfig::from_path(&p).is_err());
    let _ = fs::remove_dir_all(&dir);
}
