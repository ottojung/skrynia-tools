use super::{Authority, CompatError, CompatResult, DurableBackend, State};
use std::sync::{MutexGuard, atomic::Ordering};

impl<S, R> DurableBackend<S, R> {
    pub(crate) fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }
    pub(crate) fn checked(&self) -> CompatResult<MutexGuard<'_, State>> {
        let guard = self.lock();
        if self.uncertain.load(Ordering::SeqCst) {
            return Err(CompatError::new(503, "state_uncertain"));
        }
        Ok(guard)
    }
    pub(crate) fn authority(state: &State) -> CompatResult<Authority> {
        Authority::from_grants(state.grants.clone())
            .map_err(|e| CompatError::new(500, format!("grant_graph:{e}")))
    }
}
