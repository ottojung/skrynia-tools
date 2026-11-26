//! Userland evaluator policy; none of these budgets is enforced by the kernel.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    output: usize,
    callback: usize,
    fuel: u64,
    memory: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            output: 1 << 20,
            callback: 16 << 20,
            fuel: 100_000_000,
            memory: 64 << 20,
        }
    }
}

impl Limits {
    pub fn output(cap: usize) -> Self {
        assert!(cap > 0);
        Self {
            output: cap,
            ..Self::default()
        }
    }
    pub fn with_fuel(mut self, fuel: u64) -> Self {
        self.fuel = fuel;
        self
    }
    pub fn with_callback_cap(mut self, cap: usize) -> Self {
        self.callback = cap;
        self
    }
    pub(crate) fn output_cap(&self) -> usize {
        self.output
    }
    pub(crate) fn callback_cap(&self) -> usize {
        self.callback
    }
    pub(crate) fn fuel(&self) -> u64 {
        self.fuel
    }
    pub(crate) fn resources(&self) -> wasmi::StoreLimits {
        wasmi::StoreLimitsBuilder::new()
            .memory_size(self.memory)
            .memories(1)
            .tables(1)
            .table_elements(1 << 20)
            .instances(1)
            .build()
    }
}
