//! Userland SDK surface for the owner-approved target ABI.
//!
//! This layer follows the 0ccc90f intent records and K0's explicit
//! integration choices. Those choices are implementation assumptions,
//! not owner intent. No import is injected into evaluated children and
//! no byte transfer uses a linear-memory buffer.
mod abi;
mod decode;
mod deploy;
mod discovery;
mod fallback;
mod invoke;
mod scalar;
mod typed;
mod wire;

pub const READ1_EOF: i32 = -1;
pub const ERR_BADF: i32 = -2;
pub const ERR_INVAL: i32 = -3;
pub const ERR_UNSUPPORTED: i32 = -4;
pub const ERR_IO: i32 = -5;
pub const ERR_RANGE: i32 = -6;
pub const ERR_BOUNDS: i32 = -7;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TypedValue {
    I32(i32),
    I64(i64),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WireError {
    Version,
    Count,
    NonMinimal,
    Tag,
    Truncated,
    Trailing,
    TooLarge,
}

pub(super) fn word(bytes: &[u8], at: usize, len: usize) -> Result<u64, WireError> {
    let slice = bytes.get(at..at + len).ok_or(WireError::Truncated)?;
    let mut buf = [0u8; 8];
    buf[..len].copy_from_slice(slice);
    Ok(u64::from_le_bytes(buf))
}

pub use abi::{fixed, names};
pub use decode::decode;
pub use deploy::{DeployCheck, ImportNeed, check_imports};
pub use discovery::{DiscoveryError, parse_list, parse_version};
pub use fallback::{ByteIo, CopyFailure, Progress, copy_bytes, skip_input};
pub use invoke::{BindingError, NO_FUNCTION, check_binding};
pub use scalar::{ScalarError, cas1, read1, seek, status, write1};
pub use typed::{FixedSignature, Signature, ValType};
pub use wire::{COUNT_EXTENDED, TAG_I32, TAG_I64, VERSION, encode};
