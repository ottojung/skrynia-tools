mod admin;
mod app;
mod core;
mod precond;
mod push;
use super::take::{DecodeErr, blob, id, take};
use crate::wire::*;

pub fn parse(op: u8, src: &mut &[u8]) -> Result<Request, DecodeErr> {
    match op {
        OP_REGISTER_AUTH | OP_SET | OP_INVOKE | OP_LOOKUP | OP_REVOKE => core::parse(op, src),
        OP_APP_GET | OP_APP_PUT | OP_APP_DELETE | OP_APP_PUT_WAKE | OP_APP_DELETE_WAKE => {
            app::parse(op, src)
        }
        OP_PUSH_SUBSCRIBE | OP_PUSH_UNSUBSCRIBE | OP_PUSH_GRANT | OP_PUSH_SENDER_GRANT
        | OP_PUSH_WAKE_GRANT | OP_HINT_LIST | OP_HINT_READ | OP_HINT_ACK | OP_PUSH_SUB_FETCH
        | OP_SUB_STATUS => push::parse(op, src),
        OP_STAGE_CODE | OP_SEAL_CODE | OP_ACTIVATE_CODE | OP_GET_ACTIVE | OP_LIST_VERSIONS
        | OP_LIST_PROGRAMS => admin::parse(op, src),
        _ => Err(DecodeErr::Op),
    }
}
