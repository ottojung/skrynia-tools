//! Single supplied guest function: scalar input/output and opaque callback frames.
use super::binding::{CB_FLUSH, CB_GET, CB_PUT, FLUSH, GET, PUT, STATUS};
use super::error::{EINVAL, EOF, ERANGE};
use super::state::HostState;

pub(crate) fn dispatch(state: &mut HostState<'_, '_>, op: i32, value: i32) -> i64 {
    if let Some(code) = state.failed {
        return i64::from(code);
    }
    match op {
        GET => get(state),
        PUT => put(state, value),
        FLUSH => 0,
        CB_PUT => state.callback_put(value),
        CB_FLUSH => state.callback_flush(),
        CB_GET => state.callback_get(),
        STATUS => 0,
        _ => i64::from(EINVAL),
    }
}

fn get(state: &mut HostState<'_, '_>) -> i64 {
    match state.input.get(state.at) {
        Some(byte) => {
            state.at += 1;
            i64::from(*byte)
        }
        None => i64::from(EOF),
    }
}

fn put(state: &mut HostState<'_, '_>, value: i32) -> i64 {
    let Ok(byte) = u8::try_from(value) else {
        return state.fail(EINVAL);
    };
    if state.output.len() >= state.cap {
        return state.fail(ERANGE);
    }
    state.output.push(byte);
    0
}
