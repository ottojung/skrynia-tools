//! Failure disposition for the operator loop. Fatal classes stop the process
//! instead of spinning; transient classes get a bounded backoff.
use super::Error;

/// Fatal classes stop the process; transient classes get bounded backoff.
/// P1(11) Busy is transient lock contention. 404/410 Gone is terminal per
/// subscription, not fatal. 401/403/other status and auth/config are fatal.
pub fn fatal(e: &Error) -> bool {
    match e {
        Error::P1(11) => false,
        Error::Config(_)
        | Error::Auth(_)
        | Error::Ssrf(_)
        | Error::Crypto(_)
        | Error::Wire(_)
        | Error::P1(_)
        | Error::Quota => true,
        Error::Transport(label) => *label == "status" || label.starts_with("ledger-"),
        Error::Exhausted => false,
    }
}

/// Record `e` over an earlier failure only when it is more severe: fatal
/// classes outrank transient ones. A permanent 401/403 must never be hidden
/// by a later retryable Busy/Exhausted/I/O failure.
pub fn keep_worse(failure: &mut Option<Error>, e: Error) {
    let replace = match failure.as_ref() {
        None => true,
        Some(old) => !fatal(old) && fatal(&e),
    };
    if replace {
        *failure = Some(e);
    }
}

/// Saturating backoff: never longer than the configured cap, at least poll_ms.
pub(super) fn backoff_ms(poll_ms: u64, cap_ms: u64, streak: u32) -> u64 {
    let cap = cap_ms.max(poll_ms);
    poll_ms.saturating_mul(1u64 << streak.min(8)).min(cap)
}
