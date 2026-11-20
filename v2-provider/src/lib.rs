//! Optional *ordinary* user program translating legacy v2 semantics.
//! The Skrynia server knows only operator-provided Program 0.
mod bridge;
mod run;
pub use bridge::{Callback, InvokeBackend, InvokeError};
pub use run::{FrameError, dispatch, dispatch_bytes};

#[cfg(target_arch = "wasm32")]
mod guest;
