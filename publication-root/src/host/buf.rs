//! Private linear-memory staging for one root invocation (wasm32 only).
//!
//! A single reusable private buffer holds paths and random output. Every host
//! call copies its arguments into this buffer and validates the full `u64`
//! range before touching a descriptor; no view survives the call. Shared RAM
//! is not used for staging, so concurrent requests cannot collide.

use super::abi;

pub struct Staging {
    buf: Vec<u8>,
}

impl Staging {
    pub const fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn reset(&mut self) {
        self.buf.clear();
    }

    /// Copy `bytes` into private memory and return `(ptr, len)` for a call.
    pub fn put(&mut self, bytes: &[u8]) -> Result<(u64, u64), i32> {
        self.prepare(bytes.len())?;
        self.buf.extend_from_slice(bytes);
        Ok((self.buf.as_ptr() as u64, bytes.len() as u64))
    }

    /// Four random bytes written by the kernel into private memory.
    pub fn random(&mut self, out: &mut [u8; 4]) -> Result<(), i32> {
        self.prepare(4)?;
        self.buf.resize(4, 0);
        let ptr = self.buf.as_ptr() as u64;
        let rc = unsafe { abi::secure_random_bits(ptr, 32) };
        if rc != 0 {
            return Err(rc);
        }
        out.copy_from_slice(&self.buf[..4]);
        Ok(())
    }

    fn prepare(&mut self, len: usize) -> Result<(), i32> {
        self.buf.clear();
        // Keep a real allocation even for an empty range: a zero-length call
        // must still address a valid one-past-end position in private memory.
        self.buf.try_reserve_exact(len.max(1)).map_err(|_| -6)
    }
}
