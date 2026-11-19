mod precond;
use super::take::{DecodeErr, blob, id};
use crate::wire::{
    OP_APP_DELETE, OP_APP_GET, OP_APP_PUT, OP_INVOKE, OP_LOOKUP, OP_REGISTER, OP_REVOKE, OP_SET,
    Request,
};
use precond::{hash, precond as parse_precond};

pub fn parse(op: u8, src: &mut &[u8]) -> Result<Request, DecodeErr> {
    match op {
        OP_REGISTER => Ok(Request::Register {
            admin_cap: id(src)?,
            code: blob(src)?,
        }),
        OP_SET => Ok(Request::Set {
            admin_cap: id(src)?,
            expect: parse_precond(src)?,
            code: blob(src)?,
        }),
        OP_INVOKE => Ok(Request::Invoke {
            invoke_id: id(src)?,
            input: blob(src)?,
        }),
        OP_LOOKUP => Ok(Request::Lookup {
            invoke_id: id(src)?,
        }),
        OP_REVOKE => Ok(Request::Revoke {
            admin_cap: id(src)?,
            expect: parse_precond(src)?,
        }),
        OP_APP_GET => Ok(Request::AppGet {
            read_cap: id(src)?,
            key: blob(src)?,
        }),
        OP_APP_PUT => Ok(Request::AppPut {
            write_cap: id(src)?,
            key: blob(src)?,
            expect: parse_precond(src)?,
            value: blob(src)?,
        }),
        OP_APP_DELETE => Ok(Request::AppDelete {
            write_cap: id(src)?,
            key: blob(src)?,
            expect: hash(src)?,
        }),
        _ => Err(DecodeErr::Op),
    }
}
