mod lock;
mod random;
mod txn;
use super::data::{RandomSource, State, StateStore, decode, encode};
pub(super) use super::{Authority, CompatError, CompatResult};
use std::sync::{Mutex, atomic::AtomicBool};

pub struct DurableBackend<S, R> {
    pub(super) store: S,
    pub(super) random: R,
    pub(super) state: Mutex<State>,
    pub(super) uncertain: AtomicBool,
}
impl<S: StateStore, R: RandomSource> DurableBackend<S, R> {
    pub fn open(store: S, random: R) -> Result<Self, String> {
        let state = match store.load().map_err(|e| e.to_string())? {
            Some(bytes) => decode(&bytes)?,
            None => State::default(),
        };
        if !state.grants.is_empty() {
            Authority::from_grants(state.grants.clone()).map_err(|e| format!("grant_graph:{e}"))?;
        }
        Ok(Self {
            store,
            random,
            state: Mutex::new(state),
            uncertain: AtomicBool::new(false),
        })
    }
}
