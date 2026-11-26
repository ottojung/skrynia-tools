//! Host-side memory guard for response bytes.
//!
//! This is not an engine execution budget or resource metering: it bounds the
//! buffer the evaluator itself allocates so an adversarial guest cannot force
//! unbounded host memory growth.

/// Response-buffer bounds for one evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    output: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self { output: 1 << 20 }
    }
}

impl Limits {
    /// Set the maximum number of response bytes retained. Must be nonzero.
    pub fn output(cap: usize) -> Self {
        assert!(cap > 0, "output cap must be nonzero");
        Self { output: cap }
    }

    pub(crate) fn output_cap(&self) -> usize {
        self.output
    }
}
