//! Load the operator-selected executable into private linear memory.
use super::abi;
mod stage;
pub use stage::load;

pub const GUEST_PATH: &[u8] = b"operator/programs/root.wasm";
/// Keep the existing userland executable and framed request size limits.
pub const MAX_CODE: usize = 2_096_000;
pub const MAX_FRAME: usize = 2 * 1024 * 1024;
