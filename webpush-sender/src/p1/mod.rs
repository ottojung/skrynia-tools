//! Ordinary P1 sender-scoped operations. Opcodes 20..=23 are the only surface.
mod client;
mod decode;
mod http;
mod wire;

pub use client::{Client, P1Transport};
pub use decode::decode;
pub use http::{Edge, HttpP1};
pub use wire::{
    OP_HINT_ACK, OP_HINT_LIST, OP_HINT_READ, OP_SUB_FETCH, hint_ack, hint_list, hint_read,
    sub_fetch,
};

pub(super) use super::config::Config;
pub(super) use super::error::{Error, IoFailure, Result};
