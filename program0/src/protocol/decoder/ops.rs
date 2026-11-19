use super::take::{DecodeErr, blob, id};
use crate::wire::{
    OP_APP_GET, OP_APP_PUT, OP_INVOKE, OP_LOOKUP, OP_REGISTER, OP_REVOKE, OP_SET, Request,
};
pub fn parse(op: u8, src: &mut &[u8]) -> Result<Request, DecodeErr> {
    match op {
        OP_REGISTER => Ok(Request::Register { code: blob(src)? }),
        OP_SET => Ok(Request::Set {
            id: id(src)?,
            code: blob(src)?,
        }),
        OP_INVOKE => Ok(Request::Invoke {
            id: id(src)?,
            input: blob(src)?,
        }),
        OP_LOOKUP => Ok(Request::Lookup { id: id(src)? }),
        OP_REVOKE => Ok(Request::Revoke { id: id(src)? }),
        OP_APP_PUT => Ok(Request::AppPut {
            app: id(src)?,
            key: blob(src)?,
            value: blob(src)?,
        }),
        OP_APP_GET => Ok(Request::AppGet {
            app: id(src)?,
            key: blob(src)?,
        }),
        _ => Err(DecodeErr::Op),
    }
}
