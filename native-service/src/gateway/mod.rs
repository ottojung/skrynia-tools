//! Userland `invoke` gateway protocol for the target ordinary service.
//!
//! The evaluator supplies exactly one function to every evaluated guest:
//! `invoke(ctx: i32, op: i32, value: i32) -> i64`. `ctx` is an opaque
//! per-invocation handle, never a RAM address or a linear-memory pointer.
//! This module is target independent so host tests can exercise the exact
//! framing the compiled guest uses. No kernel ABI or policy lives here.

pub mod codes;
pub mod eval;
pub mod ops;
pub mod relay;
pub mod stream;

pub use ops::*;
pub use stream::Transport;

#[cfg(test)]
mod fake;
#[cfg(test)]
mod fake_transport;
#[cfg(test)]
mod test_relay;
#[cfg(test)]
mod test_stream;
