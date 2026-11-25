use super::cfg::RuntimeConfig;
use super::lock::Lock;
use super::os_clock::OsClock;
use super::os_dns::OsResolver;
use super::os_rng::OsRng;
use super::poll;
use super::{Error, FileLedgerStore, HttpP1, HttpsTransport, Parts, Pump, Result, VapidKey};
use std::path::Path;

pub fn run(cfg: RuntimeConfig) -> Result<()> {
    validate_ledger_dir(&cfg.ledger_dir)?;
    let _lock = Lock::acquire(&cfg.ledger_dir)?;
    let p1 = HttpP1::new(cfg.edge)?;
    let out = HttpsTransport::production();
    let key = VapidKey::from_slice(&cfg.vapid_private_key)?;
    let resolver = OsResolver;
    let rng = OsRng;
    let clock = OsClock;
    let parts = Parts::new(&cfg.pump, &p1, &out, &resolver, &key, &rng, &clock);
    let store = FileLedgerStore::from_config(&cfg.ledger_dir, &cfg.pump)?;
    let mut pump = Pump::with_https_transport(parts, store)?;
    poll::run_loop(&mut pump, cfg.poll_ms);
    #[allow(unreachable_code)]
    Ok(())
}

pub fn validate_ledger_dir(dir: &Path) -> Result<()> {
    let meta = std::fs::metadata(dir).map_err(|_| Error::Config("ledger-dir"))?;
    if !meta.is_dir() {
        return Err(Error::Config("ledger-dir"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if meta.permissions().mode() & 0o077 != 0 {
            return Err(Error::Config("ledger-dir-mode"));
        }
    }
    Ok(())
}
