//! Reusable userland P2 interpreter for synchronous P1-owned callbacks.
//!
//! The guest is an ordinary WASM module, not a hand-written mini-VM: it is
//! compiled and executed by `wasmi`. The only import is supplied here, in
//! P1's own store, so an appdata callback runs directly in P1 and no
//! suspension, P0 relay, kernel import or cross-store pointer is needed.
//!
//! See `docs/p2-interpreter.md` for the full contract and failure modes.

mod binding;
mod callback;
mod error;
mod invoke;
mod limits;
mod run;
mod state;
mod validate;

pub use binding::{Binding, CB_GET, CB_PUT, FLUSH, GET, PUT, STATUS};
pub use error::Error;
pub use limits::Limits;

use skrynia_native_service::cap::Eval;
use state::Callback;

/// A synchronous P2 evaluator usable as a native-service `Eval` capability.
#[derive(Clone, Debug)]
pub struct Interpreter {
    binding: Binding,
    limits: Limits,
}

impl Interpreter {
    /// T3/A2 binding with the default response-buffer guard.
    pub fn new() -> Self {
        Self {
            binding: Binding::p0(),
            limits: Limits::default(),
        }
    }

    /// Explicit binding and memory guard, for alternate nested contexts.
    pub fn with(binding: Binding, limits: Limits) -> Self {
        Self { binding, limits }
    }

    /// Evaluate `code` over `input`, calling `cb` synchronously for appdata.
    pub fn evaluate<'i, 'c>(
        &self,
        code: &[u8],
        input: &'i [u8],
        cb: Option<Callback<'c>>,
    ) -> Result<Vec<u8>, Error> {
        run::run(code, input, cb, &self.binding, self.limits)
    }
}

impl Eval for Interpreter {
    type Err = Error;

    fn eval(
        &self,
        wasm: &[u8],
        input: &[u8],
        cb: Option<&mut dyn FnMut(&[u8]) -> Result<Vec<u8>, i32>>,
    ) -> Result<Vec<u8>, Self::Err> {
        self.evaluate(wasm, input, cb)
    }
}

#[cfg(test)]
mod tests;
