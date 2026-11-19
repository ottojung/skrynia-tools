use super::helpers::{put_blob, put_hash, put_id};
use crate::wire::*;

fn page(out: &mut Vec<u8>, after: &Option<Vec<u8>>, limit: u16) {
    match after {
        None => out.push(0),
        Some(hash) => {
            out.push(1);
            put_hash(out, hash);
        }
    }
    out.extend_from_slice(&limit.to_le_bytes());
}
pub fn encode_admin(out: &mut Vec<u8>, req: &Request) {
    match req {
        Request::StageCode { admin_cap, code } => {
            out.push(OP_STAGE_CODE);
            put_id(out, admin_cap);
            put_blob(out, code);
        }
        Request::SealCode { admin_cap, hash } => {
            out.push(OP_SEAL_CODE);
            put_id(out, admin_cap);
            put_hash(out, hash);
        }
        Request::ActivateCode {
            admin_cap,
            expected_revision,
            target,
        } => {
            out.push(OP_ACTIVATE_CODE);
            put_id(out, admin_cap);
            out.extend_from_slice(&expected_revision.to_le_bytes());
            put_hash(out, target);
        }
        Request::GetActive { invoke_id } => {
            out.push(OP_GET_ACTIVE);
            put_id(out, invoke_id);
        }
        Request::ListVersions {
            invoke_id,
            after,
            limit,
        } => {
            out.push(OP_LIST_VERSIONS);
            put_id(out, invoke_id);
            page(out, after, *limit);
        }
        Request::ListPrograms { after, limit } => {
            out.push(OP_LIST_PROGRAMS);
            page(out, after, *limit);
        }
        _ => unreachable!(),
    }
}
