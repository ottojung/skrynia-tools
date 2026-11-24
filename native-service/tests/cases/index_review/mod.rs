//! Bounded, crash-consistent pagination and registry regressions.
mod cases1;
mod cases2;
pub mod indep_a;
pub mod indep_b;
mod support;
pub use indep_b::legacy_support;
