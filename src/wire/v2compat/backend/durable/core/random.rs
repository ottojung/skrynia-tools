use super::{CompatError, CompatResult, DurableBackend, RandomSource};
impl<S, R: RandomSource> DurableBackend<S, R> {
    pub(crate) fn random_hex(&self, bytes: usize) -> CompatResult<String> {
        let mut out = vec![0u8; bytes];
        self.random
            .fill(&mut out)
            .map_err(|e| CompatError::new(500, format!("random:{e}")))?;
        Ok(hex::encode(out))
    }
}
