use super::{Store, entry, types::AppendLog};
use anyhow::Result;

impl<S: Store> AppendLog<'_, S> {
    pub(super) fn observed_at(
        &self,
        key: &str,
        expected: &[u8],
        operation_id: &str,
    ) -> Result<Option<bool>> {
        self.store
            .get(key)?
            .map(|found| entry::observed(&found, expected, operation_id))
            .transpose()
    }
}
