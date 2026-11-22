/// Failure categories keep auth, SSRF, transport and quota distinct so callers
/// never turn an unsafe endpoint or a bad grant into a silent retry storm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Config(&'static str),
    Wire(&'static str),
    Ssrf(&'static str),
    Crypto(&'static str),
    P1(u8),
    Transport(&'static str),
    Exhausted,
    Quota,
}

/// Marker for an injected capability's own failure (transport/DNS/clock).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IoFailure;

pub type Result<T> = core::result::Result<T, Error>;
