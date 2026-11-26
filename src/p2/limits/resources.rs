use super::Limits;

impl Limits {
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
