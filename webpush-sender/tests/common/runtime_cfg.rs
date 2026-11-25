//! Hermetic runtime config and lock tests. No external network.
use std::fs;
use std::os::unix::fs::PermissionsExt;
use webpush_sender::push::proc::RuntimeConfig;

fn write_cfg(dir: &std::path::Path, extra: &str) -> std::path::PathBuf {
    let p = dir.join("cfg.json");
    let body = format!(
        r#"{{"edge":{{"ip":"127.0.0.1","port":8443,"host":"h","origin":"o","capability":"capability-012345","timeout_ms":5000,"max_request_bytes":4096,"max_response_bytes":4096}},"sender_cap":"{cap}","ns":"{cap}","key":"notice/arrive","max_payload":4096,"max_attempts":3,"base_backoff_ms":10,"max_backoff_ms":100,"ttl_secs":60,"topic":"{topic}","vapid_sub":"mailto:ops@example.invalid","ledger_epoch":0,"vapid_private_key":"{cap}","ledger_dir":"{ld}","poll_ms":1000{extra}}}"#,
        cap = "11".repeat(32),
        topic = "22".repeat(16),
        ld = dir.join("ledgers").display()
    );
    fs::write(&p, body).unwrap();
    fs::set_permissions(&p, fs::Permissions::from_mode(0o600)).unwrap();
    p
}

pub fn loads_and_rejects_bad_mode() {
    let dir = std::env::temp_dir().join(format!("rt-cfg-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("ledgers")).unwrap();
    let p = write_cfg(&dir, "");
    let cfg = RuntimeConfig::from_path(&p).unwrap();
    assert_eq!(cfg.poll_ms, 1000);
    fs::set_permissions(&p, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(RuntimeConfig::from_path(&p).is_err());
    let _ = fs::remove_dir_all(&dir);
}

pub fn lock_single_writer() {
    use webpush_sender::push::proc::Lock;
    let dir = std::env::temp_dir().join(format!("rt-lock-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
    let l1 = Lock::acquire(&dir).unwrap();
    assert!(Lock::acquire(&dir).is_err());
    drop(l1);
    let _l2 = Lock::acquire(&dir).unwrap();
    let _ = fs::remove_dir_all(&dir);
}

pub fn ledger_dir_rejects_group_other() {
    use webpush_sender::push::proc::validate_ledger_dir;
    let dir = std::env::temp_dir().join(format!("rt-dir-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(validate_ledger_dir(&dir).is_err());
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
    validate_ledger_dir(&dir).unwrap();
    let _ = fs::remove_dir_all(&dir);
}
