//! Operator-run runtime: OS capabilities, config, lock, and bounded loop.
mod cfg;
mod lock;
mod os_clock;
mod os_dns;
mod os_rng;
mod policy;
mod poll;
mod run;

pub(super) use super::{
    Clock, Config, Edge, Error, FileLedgerStore, HttpP1, HttpsTransport, IoFailure, P1Transport,
    Parts, Pump, Resolver, Result, Rng, VapidKey,
};
pub use cfg::RuntimeConfig;
pub use lock::Lock;
pub use policy::fatal;
pub use run::{run, validate_ledger_dir};

/// This process's identity without unsafe FFI: `/proc/self` is owned by the
/// process user on Linux. Used to reject foreign-owned config/lock/dir files.
#[cfg(target_os = "linux")]
pub(super) fn self_uid() -> Result<u32> {
    use std::os::unix::fs::MetadataExt;
    std::fs::metadata("/proc/self")
        .map(|m| m.uid())
        .map_err(|_| Error::Config("identity"))
}

#[cfg(not(target_os = "linux"))]
pub(super) fn self_uid() -> Result<u64> {
    Err(Error::Config("identity"))
}
