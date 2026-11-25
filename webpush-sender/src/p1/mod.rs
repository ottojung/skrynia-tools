//! Ordinary P1 sender-scoped operations. Opcodes 20..=23 are the only surface.
mod client;
mod decode;
mod http;
#[cfg(test)]
mod negatives;
mod wire;

pub use client::{Client, P1Transport};

/// Redacted edge failure class: only 401/403 are permanent auth rejections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P1Failure {
    Auth,
    Reject,
    Io,
}

impl P1Failure {
    pub fn from_status(status: u16) -> Self {
        match status {
            401 | 403 => Self::Auth,
            429 | 500..=599 => Self::Io,
            _ => Self::Reject,
        }
    }
}
pub use decode::decode;
pub use http::{Edge, HttpP1};
pub use wire::{
    OP_HINT_ACK, OP_HINT_LIST, OP_HINT_READ, OP_SUB_FETCH, hint_ack, hint_list, hint_read,
    sub_fetch,
};

pub(super) use super::config::Config;
pub(super) use super::error::{Error, Result};
