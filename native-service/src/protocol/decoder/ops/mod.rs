mod admin;
mod app;
mod core;
mod precond;
use super::take::{DecodeErr, blob, id, take};
use crate::wire::*;

pub fn parse(op: u8, src: &mut &[u8]) -> Result<Request, DecodeErr> {
    match op {
        OP_REGISTER_AUTH | OP_SET | OP_INVOKE | OP_LOOKUP | OP_REVOKE => core::parse(op, src),
        OP_APP_GET | OP_APP_PUT | OP_APP_DELETE => app::parse(op, src),
        OP_STAGE_CODE | OP_SEAL_CODE | OP_ACTIVATE_CODE | OP_GET_ACTIVE | OP_LIST_VERSIONS
        | OP_LIST_PROGRAMS => admin::parse(op, src),
        _ => Err(DecodeErr::Op),
    }
}
