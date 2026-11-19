use super::data::{RandomSource, State, StateStore, decode, encode};
use super::{Authority, CompatError, CompatResult};
use std::sync::{Mutex, MutexGuard};

pub struct DurableBackend<S, R> {
    pub(super) store: S,
    pub(super) random: R,
    state: Mutex<State>,
}

impl<S: StateStore, R: RandomSource> DurableBackend<S, R> {
    pub fn open(store: S, random: R) -> Result<Self, String> {
        let state = match store.load().map_err(|e| e.to_string())? {
            Some(bytes) => decode(&bytes)?,
            None => State::default(),
        };
        Ok(Self {
            store,
            random,
            state: Mutex::new(state),
        })
    }
}
impl<S, R> DurableBackend<S, R> {
    pub(super) fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }
    pub(super) fn authority(state: &State) -> CompatResult<Authority> {
        Authority::from_grants(state.grants.clone())
            .map_err(|e| CompatError::new(500, format!("grant_graph:{e}")))
    }
}
impl<S: StateStore, R> DurableBackend<S, R> {
    pub(super) fn persist(&self, state: &State) -> CompatResult<()> {
        let bytes =
            encode(state).map_err(|e| CompatError::new(500, format!("state_encode:{e}")))?;
        self.store
            .save(&bytes)
            .map_err(|e| CompatError::new(500, format!("state_save:{e}")))
    }
}
impl<S, R: RandomSource> DurableBackend<S, R> {
    pub(super) fn random_hex(&self, bytes: usize) -> CompatResult<String> {
        let mut out = vec![0u8; bytes];
        self.random
            .fill(&mut out)
            .map_err(|e| CompatError::new(500, format!("random:{e}")))?;
        Ok(hex::encode(out))
    }
}
