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

pub(super) fn take(src: &[u8], pos: &mut usize, dst: &mut [u8]) -> usize {
    let n = src.len().saturating_sub(*pos).min(dst.len());
    dst[..n].copy_from_slice(&src[*pos..*pos + n]);
    *pos += n;
    n
}
