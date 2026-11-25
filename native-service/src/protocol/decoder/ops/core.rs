use super::precond::precond as parse_precond;
use super::*;
pub fn parse(op: u8, src: &mut &[u8]) -> Result<Request, DecodeErr> {
    match op {
        crate::wire::OP_REGISTER_AUTH => Ok(Request::Register {
            operator_cap: id(src)?,
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
        OP_LOOKUP => Ok(Request::Lookup(id(src)?)),
        OP_REVOKE => Ok(Request::Revoke(id(src)?, parse_precond(src)?)),
        _ => Err(DecodeErr::Op),
    }
}
