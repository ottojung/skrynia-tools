//! Per-invocation host state. Every field is owned by one evaluation; no
//! global RAM page, native mutex or cross-store pointer is involved.

use super::limits::Limits;

/// Callback capability from the existing `Eval` trait, executed in P1.
pub(crate) type Callback<'a> = &'a mut dyn FnMut(&[u8]) -> Result<Vec<u8>, i32>;

pub(crate) struct HostState<'i, 'c> {
    pub(crate) input: &'i [u8],
    pub(crate) at: usize,
    pub(crate) output: Vec<u8>,
    pub(crate) cap: usize,
    pub(crate) cb: Option<Callback<'c>>,
    pub(crate) reply: Vec<u8>,
    pub(crate) reply_at: usize,
    pub(crate) failed: Option<i32>,
}

impl<'i, 'c> HostState<'i, 'c> {
    pub(crate) fn new(input: &'i [u8], cb: Option<Callback<'c>>, limits: Limits) -> Self {
        Self {
            input,
            at: 0,
            output: Vec::new(),
            cap: limits.output_cap(),
            cb,
            reply: Vec::new(),
            reply_at: 0,
            failed: None,
        }
    }

    pub(crate) fn failed(&self) -> Option<i32> {
        self.failed
    }
    pub(crate) fn set_failed(&mut self, code: i32) {
        if self.failed.is_none() {
            self.failed = Some(code);
        }
    }
    pub(crate) fn finish(self) -> Result<Vec<u8>, i32> {
        match self.failed {
            Some(code) => Err(code),
            None => Ok(self.output),
        }
    }
}
