use super::*;

pub fn parse(op: u8, src: &mut &[u8]) -> Result<crate::wire::Request, DecodeErr> {
    use crate::wire::{OP_PUSH_SUBSCRIBE, OP_PUSH_UNSUBSCRIBE, Request};
    match op {
        OP_PUSH_SUBSCRIBE => Ok(Request::PushSubscribe(
            id(src)?,
            blob(src)?,
            blob(src)?,
            blob(src)?,
        )),
        OP_PUSH_UNSUBSCRIBE => Ok(Request::PushUnsubscribe(id(src)?, blob(src)?, blob(src)?)),
        _ => Err(DecodeErr::Op),
    }
}
