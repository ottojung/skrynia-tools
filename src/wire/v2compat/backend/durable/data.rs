use super::Grant;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub trait StateStore {
    type Error: std::fmt::Display;
    fn load(&self) -> Result<Option<Vec<u8>>, Self::Error>;
    fn save(&self, bytes: &[u8]) -> Result<(), Self::Error>;
}
pub trait RandomSource {
    type Error: std::fmt::Display;
    fn fill(&self, out: &mut [u8]) -> Result<(), Self::Error>;
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct State {
    #[serde(default)]
    pub grants: Vec<Grant>,
    #[serde(default)]
    pub objects: BTreeMap<String, Vec<u8>>,
    #[serde(default)]
    pub next_grant: u64,
    #[serde(default)]
    pub next_user: u64,
}
pub fn encode(state: &State) -> Result<Vec<u8>, String> {
    serde_json::to_vec(state).map_err(|e| e.to_string())
}
pub fn decode(bytes: &[u8]) -> Result<State, String> {
    serde_json::from_slice(bytes).map_err(|e| e.to_string())
}
