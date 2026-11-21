use super::ExecError;
use super::io::{hash32, invoke_req};
use crate::{
    CAS_RETRIES, Entry, STATE_KEY, STATUS_OK, app_get, app_put_absent, app_put_present, apply,
};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

pub fn cas_put<F>(
    cap: &[u8],
    entry: &Entry,
    fragments: BTreeMap<String, String>,
    updated_at: Option<String>,
    invoke: &mut F,
) -> Result<(Vec<u8>, bool, usize), ExecError>
where
    F: FnMut(&[u8], &mut [u8]) -> i32,
{
    for retry in 0..CAS_RETRIES {
        let (st, body) = invoke_req(invoke, &app_get(cap, STATE_KEY))?;
        let current = if st == STATUS_OK {
            Some(body)
        } else if st == 1 && body.first() == Some(&8) {
            None
        } else {
            return Err(ExecError::Io);
        };
        let (next, created) = apply(
            current.as_deref(),
            entry.clone(),
            fragments.clone(),
            updated_at.clone(),
        )
        .map_err(ExecError::Rejected)?;
        let req = match current.as_deref() {
            Some(b) => app_put_present(cap, STATE_KEY, &hash32(b), &next),
            None => app_put_absent(cap, STATE_KEY, &next),
        };
        let (pst, pbody) = invoke_req(invoke, &req)?;
        if pst == STATUS_OK {
            return Ok((next, created, retry));
        }
        if pst == 1 && (pbody.first() == Some(&10) || pbody.first() == Some(&11)) {
            continue;
        }
        return Err(ExecError::Io);
    }
    Err(ExecError::Busy)
}
