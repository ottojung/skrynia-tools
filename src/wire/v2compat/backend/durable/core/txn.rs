use super::encode;
use super::{CompatError, CompatResult, DurableBackend, State, StateStore};
use std::sync::atomic::Ordering;

impl<S: StateStore, R> DurableBackend<S, R> {
    pub(crate) fn change<T>(
        &self,
        edit: impl FnOnce(&mut State) -> CompatResult<T>,
    ) -> CompatResult<T> {
        let mut stored = self.checked()?;
        let mut next = stored.clone();
        let value = edit(&mut next)?;
        if !next.grants.is_empty() {
            super::Authority::from_grants(next.grants.clone())
                .map_err(|e| CompatError::new(500, format!("grant_graph:{e}")))?;
        }
        let bytes =
            encode(&next).map_err(|e| CompatError::new(500, format!("state_encode:{e}")))?;
        if self.store.save(&bytes).is_err() {
            self.uncertain.store(true, Ordering::SeqCst);
            return Err(CompatError::new(503, "state_uncertain"));
        }
        *stored = next;
        Ok(value)
    }
}
