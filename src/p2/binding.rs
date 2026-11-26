//! The frozen nested-P2 binding: one supplied function inside P1's own store.
//!
//! This is a userland convention only. No kernel import, implicit authority,
//! linear-memory pointer or P1/table index crosses between stores.

/// Import pair and opaque context handle supplied to a P2 guest.
///
/// The handle is never a RAM address or a function index. A negative handle is
/// reserved and is rejected by the host function with `EBADF`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Binding {
    module: String,
    function: String,
    context: i32,
}

impl Binding {
    /// The T3/A2 convention: `p0.invoke`, context handle 1, selector 0.
    pub fn p0() -> Self {
        Self {
            module: "p0".to_string(),
            function: "invoke".to_string(),
            context: 1,
        }
    }

    /// Replace the opaque context handle for a nested invocation.
    pub fn with_context(mut self, context: i32) -> Self {
        self.context = context;
        self
    }

    pub(crate) fn module(&self) -> &str {
        &self.module
    }
    pub(crate) fn function(&self) -> &str {
        &self.function
    }
    pub(crate) fn context(&self) -> i32 {
        self.context
    }
}

impl Default for super::Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

/// Next input byte (`0..=255`), negative on EOF.
pub const GET: i32 = 0;
/// Append one response byte (`0..=255`).
pub const PUT: i32 = 1;
/// Accepted no-op; output is accumulated in the evaluator.
pub const FLUSH: i32 = 2;
/// Drain one byte from the complete P1 callback reply.
pub const CB_GET: i32 = 3;
/// Append one byte to the opaque P1 callback request.
pub const CB_PUT: i32 = 4;
/// Status word; zero means ready.
pub const STATUS: i32 = 5;
/// Execute the complete pending callback frame exactly once.
pub const CB_FLUSH: i32 = 6;
