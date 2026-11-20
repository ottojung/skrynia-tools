use super::page::page;
use super::*;
pub fn encode_admin(out: &mut Vec<u8>, req: &AdminRequest) {
    match req {
        AdminRequest::StageCode { admin_cap, code } => {
            out.push(OP_STAGE_CODE);
            put_id(out, admin_cap);
            put_blob(out, code);
        }
        AdminRequest::SealCode { admin_cap, hash } => {
            out.push(OP_SEAL_CODE);
            put_id(out, admin_cap);
            put_hash(out, hash);
        }
        AdminRequest::ActivateCode {
            admin_cap,
            expected_revision,
            target,
        } => {
            out.push(OP_ACTIVATE_CODE);
            put_id(out, admin_cap);
            out.extend_from_slice(&expected_revision.to_le_bytes());
            put_hash(out, target);
        }
        AdminRequest::GetActive { invoke_id } => {
            out.push(OP_GET_ACTIVE);
            put_id(out, invoke_id);
        }
        AdminRequest::ListVersions {
            invoke_id,
            after,
            limit,
        } => {
            out.push(OP_LIST_VERSIONS);
            put_id(out, invoke_id);
            page(out, after, *limit);
        }
        AdminRequest::ListPrograms { after, limit } => {
            out.push(OP_LIST_PROGRAMS);
            page(out, after, *limit);
        }
    }
}
