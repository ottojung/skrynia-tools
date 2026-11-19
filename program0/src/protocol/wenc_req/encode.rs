use super::encode_app::encode_app;
use super::helpers::{put_blob, put_id, put_precond};
use crate::wire::{OP_INVOKE, OP_LOOKUP, OP_REGISTER, OP_REVOKE, OP_SET, Request};

pub fn encode_body(out: &mut Vec<u8>, req: &Request) {
    match req {
        Request::Register { admin_cap, code } => {
            out.push(OP_REGISTER);
            put_id(out, admin_cap);
            put_blob(out, code);
        }
        Request::Set {
            admin_cap,
            expect,
            code,
        } => {
            out.push(OP_SET);
            put_id(out, admin_cap);
            put_precond(out, expect);
            put_blob(out, code);
        }
        Request::Invoke { invoke_id, input } => {
            out.push(OP_INVOKE);
            put_id(out, invoke_id);
            put_blob(out, input);
        }
        Request::Lookup { invoke_id } => {
            out.push(OP_LOOKUP);
            put_id(out, invoke_id);
        }
        Request::Revoke { admin_cap, expect } => {
            out.push(OP_REVOKE);
            put_id(out, admin_cap);
            put_precond(out, expect);
        }
        Request::StageCode { .. }
        | Request::SealCode { .. }
        | Request::ActivateCode { .. }
        | Request::GetActive { .. }
        | Request::ListVersions { .. }
        | Request::ListPrograms { .. } => super::encode_admin::encode_admin(out, req),
        _ => encode_app(out, req),
    }
}
