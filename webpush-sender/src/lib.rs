//! Standalone least-privilege outbound WebPush sender.
//! Kernel, Program 0, P1 and v2 routes are untouched.
#![forbid(unsafe_code)]

#[cfg(test)]
mod source_lints;

pub mod config;
pub mod error;
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
