//! Parse discovery shapes from /syscalls_list and /version.
mod list;
mod ver;
use super::abi::MODULE;
use super::typed::{Signature, ValType};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscoveryError {
    MissingKey,
    MissingAbi,
    BadAbi,
    BadShape,
    BadType,
    DuplicateName,
}

pub use list::parse_list;
pub use ver::{parse_version, version_abi};
