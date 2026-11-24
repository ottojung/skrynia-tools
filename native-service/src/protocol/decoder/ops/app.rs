use super::precond::{hash, precond as parse_precond};
use super::*;
pub fn parse(op: u8, src: &mut &[u8]) -> Result<Request, DecodeErr> {
    match op {
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
        OP_APP_PUT_WAKE => Ok(Request::AppPutWake(
            id(src)?,
            blob(src)?,
            parse_precond(src)?,
            blob(src)?,
            id(src)?,
            blob(src)?,
        )),
        OP_APP_DELETE_WAKE => Ok(Request::AppDeleteWake(
            id(src)?,
            blob(src)?,
            hash(src)?,
            id(src)?,
            blob(src)?,
        )),
        _ => Err(DecodeErr::Op),
    }
}
