use super::take::{DecodeErr, blob, fixed32, id, take};
use crate::wire::{Request, V2Kind};

pub fn bootstrap(src: &mut &[u8]) -> Result<Request, DecodeErr> {
    Ok(Request::V2Bootstrap {
        token: fixed32(src)?,
    })
}

pub fn mutate(src: &mut &[u8]) -> Result<Request, DecodeErr> {
    let token = fixed32(src)?;
    let key = id(src)?;
    let kind = match take(src, 1)?[0] {
        1 => V2Kind::Create,
        2 => V2Kind::Replace,
        3 => V2Kind::Delete,
        _ => return Err(DecodeErr::Op),
    };
    let flags = take(src, 1)?[0];
    let expected = if flags & 1 != 0 {
        Some(fixed32(src)?)
    } else {
        None
    };
    let body = blob(src)?;
    Ok(Request::V2Mutate {
        token,
        key,
        kind,
        expected,
        body,
    })
}
