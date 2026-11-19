use super::{DecodeErr, Request, blob, id, precond::hash};
use crate::wire::*;

fn page(src: &mut &[u8]) -> Result<(Option<Vec<u8>>, u16), DecodeErr> {
    let after = match super::super::take::take(src, 1)?[0] {
        0 => None,
        1 => Some(hash(src)?),
        _ => return Err(DecodeErr::Op),
    };
    let n: [u8; 2] = super::super::take::take(src, 2)?.try_into().unwrap();
    let limit = u16::from_le_bytes(n);
    if limit == 0 || limit > 256 {
        return Err(DecodeErr::Op);
    }
    Ok((after, limit))
}
pub fn parse(op: u8, src: &mut &[u8]) -> Result<Request, DecodeErr> {
    Ok(match op {
        OP_STAGE_CODE => Request::StageCode {
            admin_cap: id(src)?,
            code: blob(src)?,
        },
        OP_SEAL_CODE => Request::SealCode {
            admin_cap: id(src)?,
            hash: hash(src)?,
        },
        OP_ACTIVATE_CODE => {
            let admin_cap = id(src)?;
            let b: [u8; 8] = super::super::take::take(src, 8)?.try_into().unwrap();
            Request::ActivateCode {
                admin_cap,
                expected_revision: u64::from_le_bytes(b),
                target: hash(src)?,
            }
        }
        OP_GET_ACTIVE => Request::GetActive {
            invoke_id: id(src)?,
        },
        OP_LIST_VERSIONS => {
            let invoke_id = id(src)?;
            let (after, limit) = page(src)?;
            Request::ListVersions {
                invoke_id,
                after,
                limit,
            }
        }
        OP_LIST_PROGRAMS => {
            let (after, limit) = page(src)?;
            Request::ListPrograms { after, limit }
        }
        _ => return Err(DecodeErr::Op),
    })
}
