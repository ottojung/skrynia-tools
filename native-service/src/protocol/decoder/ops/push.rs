use super::*;

fn seq(src: &mut &[u8]) -> Result<u64, DecodeErr> {
    let raw = take(src, 8)?;
    Ok(u64::from_le_bytes(raw.try_into().unwrap()))
}

pub fn parse(op: u8, src: &mut &[u8]) -> Result<crate::wire::Request, DecodeErr> {
    use crate::wire::{
        OP_HINT_ACK, OP_HINT_LIST, OP_HINT_READ, OP_PUSH_GRANT, OP_PUSH_SENDER_GRANT,
        OP_PUSH_SUB_FETCH, OP_PUSH_SUBSCRIBE, OP_PUSH_UNSUBSCRIBE, Request,
    };
    match op {
        OP_PUSH_SUBSCRIBE => Ok(Request::PushSubscribe(
            id(src)?,
            blob(src)?,
            blob(src)?,
            blob(src)?,
        )),
        OP_PUSH_UNSUBSCRIBE => Ok(Request::PushUnsubscribe(id(src)?, blob(src)?, blob(src)?)),
        OP_PUSH_GRANT => Ok(Request::PushGrant(
            id(src)?,
            id(src)?,
            blob(src)?,
            blob(src)?,
        )),
        OP_PUSH_SENDER_GRANT => Ok(Request::PushSenderGrant(id(src)?, id(src)?, blob(src)?)),
        OP_HINT_LIST => Ok(Request::HintList(id(src)?, id(src)?, blob(src)?)),
        OP_HINT_READ => Ok(Request::HintRead(id(src)?, id(src)?, blob(src)?, seq(src)?)),
        OP_HINT_ACK => Ok(Request::HintAck(id(src)?, id(src)?, blob(src)?, seq(src)?)),
        OP_PUSH_SUB_FETCH => Ok(Request::SubFetch(id(src)?, id(src)?, blob(src)?)),
        _ => Err(DecodeErr::Op),
    }
}
