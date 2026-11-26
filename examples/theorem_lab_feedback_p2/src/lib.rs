extern crate alloc;

pub mod abi;
pub mod contract;
pub mod guest;
pub mod merge;
pub mod wire;

#[cfg(target_arch = "wasm32")]
pub use abi::{CB_FLUSH, CB_GET, CB_PUT, GET, PUT, byte, call};
pub use contract::{
    CAS_RETRIES, MAX_CHAIN, MAX_FEEDBACK, MAX_FRAGMENT, MAX_FRAGMENTS, MAX_ID, MAX_INPUT,
    MAX_STATE, MAX_TEXT, OP_SUBMIT, STATE_KEY, VERSION,
};
pub use guest::{ExecError, submit_json};
pub use merge::{Entry, State, apply, validate_entry};
pub use wire::{STATUS_ERR, STATUS_OK, app_get, app_put_absent, app_put_present, decode_status};
