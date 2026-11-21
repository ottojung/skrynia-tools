use super::*;
pub fn parse(op: u8, src: &mut &[u8]) -> Result<Request, DecodeErr> {
    Ok(Request::Admin(match op {
        OP_STAGE_CODE => AdminRequest::StageCode {
            admin_cap: id(src)?,
            code: blob(src)?,
        },
        OP_SEAL_CODE => AdminRequest::SealCode {
            admin_cap: id(src)?,
            hash: hash(src)?,
        },
        OP_ACTIVATE_CODE => {
            let admin_cap = id(src)?;
            let b: [u8; 8] = take(src, 8)?.try_into().unwrap();
            AdminRequest::ActivateCode {
                admin_cap,
                expected_revision: u64::from_le_bytes(b),
                target: hash(src)?,
            }
        }
        OP_GET_ACTIVE => AdminRequest::GetActive {
            invoke_id: id(src)?,
        },
        OP_LIST_VERSIONS => {
            let invoke_id = id(src)?;
            let (after, limit) = page::page(src)?;
            AdminRequest::ListVersions {
                invoke_id,
                after,
                limit,
            }
        }
        OP_LIST_PROGRAMS => {
            let (after, limit) = page::page(src)?;
            AdminRequest::ListPrograms { after, limit }
        }
        _ => return Err(DecodeErr::Op),
    }))
}
