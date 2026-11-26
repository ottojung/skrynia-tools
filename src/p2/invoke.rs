//! The single supplied guest function: scalar bytes only.
//!
//! A sticky failure prevents a second callback after an uncertain reply:
//! the guest may keep calling, but P1's callback is never replayed.

use super::binding::{CB_GET, CB_PUT, FLUSH, GET, PUT, STATUS};
use super::error::{EINVAL, EIO};
use super::state::HostState;

pub(crate) fn dispatch(state: &mut HostState<'_, '_>, op: i32, value: i32) -> i64 {
    if let Some(code) = state.failed() {
        return i64::from(code);
    }
    match op {
        GET => state.get(),
        PUT => state.put(value),
        FLUSH => 0,
        CB_GET => get(state, value),
        CB_PUT => state.reply_byte(),
        STATUS => 0,
        _ => i64::from(EINVAL),
    }
}

fn get(state: &mut HostState<'_, '_>, value: i32) -> i64 {
    if state.reply_pending() {
        return i64::from(EIO);
    }
    let Ok(byte) = u8::try_from(value) else {
        return i64::from(EINVAL);
    };
    match state.invoke_callback(byte) {
        Ok(()) => state.reply_byte(),
        Err(code) => {
            state.set_failed(code);
            i64::from(code)
        }
    }
}
