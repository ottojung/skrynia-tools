//! Private HTTP provider for the `publish` and `unpublish` Skrynia syscalls.
//!
//! The kernel only sees a numeric ticket. Path data crosses the boundary
//! through an operator-private mailbox that is read-only here; source bytes
//! are never served by this service.

pub mod abi;
mod boundary;
pub mod config;
mod fs;
mod identity;
pub mod mailbox;
pub mod path;
pub mod roots;
pub mod server;

pub use fs::State;
