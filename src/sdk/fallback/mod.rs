//! Userland byte-I/O reference helpers for optional derived operations.
//!
//! This is a software-provider helper over read1/write1, not the unresolved
//! four-integer `copy` FFI: no copy signature is invented here.
use super::scalar::{ScalarError, read1, seek, write1};

/// Injected byte-only descriptor operations (no RAM buffer, no bulk import).
pub trait ByteIo {
    fn read1(&mut self, fd: i32) -> i32;
    fn write1(&mut self, fd: i32, byte: u32) -> i32;
    fn seek(&mut self, fd: i32, shift: i64) -> i64;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CopyFailure {
    Read(ScalarError),
    Write(ScalarError),
    Seek(ScalarError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Progress {
    pub bytes: u64,
    pub eof: bool,
}

/// Forward byte pump. A write error keeps bytes already read; never replay.
pub fn copy_bytes<I: ByteIo>(io: &mut I, src: i32, dst: i32) -> Result<Progress, CopyFailure> {
    let mut bytes = 0u64;
    loop {
        let raw = io.read1(src);
        if raw == super::READ1_EOF {
            return Ok(Progress { bytes, eof: true });
        }
        let byte = read1(raw).map_err(CopyFailure::Read)?;
        write1(io.write1(dst, u32::from(byte))).map_err(CopyFailure::Write)?;
        bytes += 1;
    }
}

/// Relative input skip. K0 clamps a short forward skip at EOF.
pub fn skip_input<I: ByteIo>(io: &mut I, fd: i32, shift: i64) -> Result<u64, CopyFailure> {
    seek(io.seek(fd, shift)).map_err(CopyFailure::Seek)
}

mod buffered;
pub use buffered::{BufferedCopyFailure, BulkIo, copy_buffered};
