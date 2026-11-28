pub(super) use super::{auth, bridge, frame, host::Host, path};
pub(super) use fake::Fake;

mod cases;
mod cases2;
mod cases3;
mod cases4;
mod cases5;
mod cases6;
mod fake;
mod host;
mod open_error;

pub(super) fn with_io_error(read: Option<i32>, open: Option<i32>) -> Fake {
    Fake {
        read_error: read,
        verifier_open_error: open,
        ..Fake::default()
    }
}

pub(super) fn take(src: &[u8], pos: &mut usize, dst: &mut [u8]) -> usize {
    let n = src.len().saturating_sub(*pos).min(dst.len());
    dst[..n].copy_from_slice(&src[*pos..*pos + n]);
    *pos += n;
    n
}

pub(super) fn with_verifier(token: &[u8; 32]) -> Fake {
    use sha2::{Digest, Sha256};
    Fake {
        verifier: Sha256::digest(token).into(),
        ..Fake::default()
    }
}
pub(super) fn record_invoke(fake: &mut Fake, publish: bool, ticket: u32) -> Option<i32> {
    if fake.trap {
        return None;
    }
    fake.invokes.push((publish, ticket));
    Some(fake.provider)
}
