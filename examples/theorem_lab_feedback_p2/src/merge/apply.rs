use super::{Entry, State, validate_entry};
use crate::{MAX_FEEDBACK, MAX_STATE};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

pub fn apply(
    current: Option<&[u8]>,
    entry: Entry,
    fragments: BTreeMap<String, String>,
    updated_at: Option<String>,
) -> Result<(Vec<u8>, bool), &'static str> {
    let mut state = match current {
        Some(b) => serde_json::from_slice::<State>(b).map_err(|_| "bad_state")?,
        None => State::empty(),
    };
    if state.version != 1 {
        return Err("bad_state_version");
    }
    validate_entry(&entry, &fragments)?;
    for (k, v) in fragments {
        state.html_fragments.entry(k).or_insert(v);
    }
    let existed = state.feedback.iter().any(|e| e.id == entry.id);
    if !existed {
        if state.feedback.len() >= MAX_FEEDBACK {
            return Err("quota_feedback");
        }
        state.feedback.push(entry);
    }
    if updated_at.as_ref().is_some_and(|s| s.len() <= 64) {
        state.updated_at = updated_at;
    }
    let bytes = serde_json::to_vec(&state).map_err(|_| "encode_state")?;
    if bytes.len() > MAX_STATE {
        return Err("quota_state");
    }
    Ok((bytes, !existed))
}
