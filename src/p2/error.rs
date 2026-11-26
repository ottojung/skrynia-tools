//! Typed evaluator failures. Kept distinct so callers can tell malformed
//! guests, link failures, traps and callback failures apart.

/// Failure of a userland P2 evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// Validation rejected the module bytes.
    InvalidWasm,
    /// The required single `invoke` import is absent.
    MissingImport,
    /// More than one import was requested; only one is supplied.
    ExtraImport,
    /// Import pair or ordered parameter/result types do not match.
    WrongImport,
    /// Exported `_start () -> ()` is absent.
    MissingStart,
    /// Exported `_start` has the wrong signature.
    WrongStart,
    /// The supplied function could not be linked.
    Link,
    /// Instantiation or guest execution trapped.
    Trap,
    /// Userland instruction budget exhausted.
    Budget,
    /// A callback op was used without a callback capability.
    NoCallback,
    /// A P1-owned callback returned an error code; failure is sticky.
    Callback(i32),
}

pub(crate) const EOF: i32 = -1;
pub(crate) const EBADF: i32 = -2;
pub(crate) const EINVAL: i32 = -3;
pub(crate) const ENOTSUP: i32 = -4;
pub(crate) const EIO: i32 = -5;
pub(crate) const ERANGE: i32 = -6;

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidWasm => f.write_str("invalid wasm module"),
            Self::MissingImport => f.write_str("missing supplied-function import"),
            Self::ExtraImport => f.write_str("more than one import"),
            Self::WrongImport => f.write_str("import pair or signature mismatch"),
            Self::MissingStart => f.write_str("missing exported _start"),
            Self::WrongStart => f.write_str("_start signature mismatch"),
            Self::Link => f.write_str("host function link failure"),
            Self::Trap => f.write_str("guest trap"),
            Self::Budget => f.write_str("userland instruction budget exhausted"),
            Self::NoCallback => f.write_str("no callback capability"),
            Self::Callback(code) => write!(f, "P1 callback failed with {code}"),
        }
    }
}
