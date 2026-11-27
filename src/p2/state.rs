//! Per-invocation host state: no global request state or shared pointers.
use super::limits::Limits;

pub(crate) type Callback<'a> = &'a mut dyn FnMut(&[u8]) -> Result<Vec<u8>, i32>;

pub(crate) struct HostState<'i, 'c> {
    pub(crate) input: &'i [u8],
    pub(crate) at: usize,
    pub(crate) output: Vec<u8>,
    pub(crate) cap: usize,
    pub(crate) cb: Option<Callback<'c>>,
    pub(crate) request: Vec<u8>,
    pub(crate) reply: Vec<u8>,
    pub(crate) reply_at: usize,
    pub(crate) callback_cap: usize,
    pub(crate) resources: wasmi::StoreLimits,
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
            request: Vec::new(),
            reply: Vec::new(),
            reply_at: 0,
            callback_cap: limits.callback_cap(),
            resources: limits.resources(),
            failed: None,
        }
    }
    pub(crate) fn fail(&mut self, code: i32) -> i64 {
        let code = *self.failed.get_or_insert(code);
        i64::from(code)
    }
    pub(crate) fn finish(self) -> Result<Vec<u8>, i32> {
        if let Some(code) = self.failed {
            return Err(code);
        }
        if !self.request.is_empty() || self.reply_at < self.reply.len() {
            return Err(super::error::EIO);
        }
        Ok(self.output)
    }
}
