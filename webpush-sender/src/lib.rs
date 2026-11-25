//! Standalone least-privilege outbound WebPush sender.
//! Kernel, Program 0, P1 and v2 routes are untouched.
#![forbid(unsafe_code)]

#[cfg(test)]
mod source_lints;

pub mod config;
pub mod error {
    /// Failure categories keep auth, SSRF, transport and quota distinct so callers
    /// never turn an unsafe endpoint or a bad grant into a silent retry storm.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Error {
        Config(&'static str),
        Wire(&'static str),
        Ssrf(&'static str),
        Crypto(&'static str),
        P1(u8),
        Transport(&'static str),
        Exhausted,
        Quota,
    }

    /// Marker for an injected capability's own failure (transport/DNS/clock).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct IoFailure;

    pub type Result<T> = core::result::Result<T, Error>;
}
pub mod ledger;
pub mod p1;
pub mod pump;
pub mod push;
pub mod sub;
pub mod vapid;

pub use config::Config;
pub use error::{Error, Result};
pub use pump::{Pump, Report};

#[cfg(test)]
mod structure {
    #[test]
    fn canonical_seven_lints_and_tree_bounds() {
        let root = std::env::current_dir().expect("cwd");
        super::source_lints::check(&root);
    }
}
