use super::*;

pub fn parse(op: u8, src: &mut &[u8]) -> Result<crate::wire::Request, DecodeErr> {
    use crate::wire::{OP_PUSH_SUBSCRIBE, OP_PUSH_UNSUBSCRIBE, Request};
    match op {
        OP_PUSH_SUBSCRIBE => Ok(Request::PushSubscribe {
            write_cap: id(src)?,
            key: blob(src)?,
            subscriber: blob(src)?,
            subscription: blob(src)?,
        }),
        OP_PUSH_UNSUBSCRIBE => Ok(Request::PushUnsubscribe {
            write_cap: id(src)?,
            key: blob(src)?,
            subscriber: blob(src)?,
        }),
        _ => Err(DecodeErr::Op),
    }
}
