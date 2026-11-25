use super::fakes::{FakeP1, FakeResolver, FixedRng, ManualClock, ScriptedTransport};
use webpush_sender::{Config, Pump, vapid::VapidKey};

pub fn build<'a>(
    cfg: &'a Config,
    p1: &'a FakeP1,
    out: &'a ScriptedTransport,
    r: &'a FakeResolver,
    key: &'a VapidKey,
    rng: &'a FixedRng,
    clock: &'a ManualClock,
) -> Pump<'a, FakeP1, ScriptedTransport, FakeResolver, FixedRng, ManualClock> {
    Pump::new(cfg, p1, out, r, key, rng, clock).unwrap()
}

pub fn res(status: u8, body: &[u8]) -> Vec<u8> {
    let mut out = vec![1u8, status];
    out.extend_from_slice(&(body.len() as u32).to_le_bytes());
    out.extend_from_slice(body);
    out
}

pub fn stale_tmp_and_symlink_safe() {
    use super::fixtures::config;
    use webpush_sender::ledger::{FileLedgerStore, LedgerStore};
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
    let _ = std::fs::remove_dir_all(&dir);
}
