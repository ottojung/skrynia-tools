//! Portable bounded forward transfer, not the named copy FFI signature.
use super::{CopyFailure, Progress, ScalarError};
mod transfer;

pub trait BulkIo {
    fn read(&mut self, fd: i32, buf: &mut [u8]) -> i64;
    fn write(&mut self, fd: i32, buf: &[u8]) -> i64;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BufferedCopyFailure {
    pub error: CopyFailure,
    pub consumed: u64,
    pub written: u64,
}

/// Copy from current descriptor cursors, at most limit bytes, via caller scratch.
/// Requires distinct handles and non-overlapping ranges. Not a memmove.
/// On failure consumed may exceed written; never retry from the start.
pub fn copy_buffered<I: BulkIo>(
    io: &mut I,
    src: i32,
    dst: i32,
    limit: u64,
    scratch: &mut [u8],
) -> Result<Progress, BufferedCopyFailure> {
    let (mut consumed, mut written) = (0, 0);
    if src == dst || (limit > 0 && scratch.is_empty()) {
        return Err(BufferedCopyFailure {
            error: CopyFailure::Read(ScalarError(-3)),
            consumed,
            written,
        });
    }
    while consumed < limit {
        let cap = (limit - consumed).min(scratch.len() as u64) as usize;
        let n = io.read(src, &mut scratch[..cap]);
        let count = transfer::count(n, cap).map_err(|e| BufferedCopyFailure {
            error: CopyFailure::Read(e),
            consumed,
            written,
        })?;
        if count == 0 {
            return Ok(Progress {
                bytes: written,
                eof: true,
            });
        }
        consumed += count as u64;
        transfer::write_chunk(io, dst, &scratch[..count], consumed, &mut written)?;
    }
    Ok(Progress {
        bytes: written,
        eof: false,
    })
}
