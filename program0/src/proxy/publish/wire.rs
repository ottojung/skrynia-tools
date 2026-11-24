pub const OP_PUBLISH: u8 = 16;
pub const OP_UNPUBLISH: u8 = 17;
const INVALID: i32 = -1;

// Canonical wire builder; the ordinary-service side mirrors these bytes.
#[allow(dead_code)]
pub fn request(op: u8, rel: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(5 + rel.len());
    out.push(op);
    out.extend_from_slice(&(rel.len() as u32).to_le_bytes());
    out.extend_from_slice(rel);
    out
}

fn invalid() -> Vec<u8> {
    INVALID.to_le_bytes().to_vec()
}

/// Reply shape matches the generic mediator: i32 callback status, then data.
pub fn accepted(code: u32) -> Vec<u8> {
    let mut out = 0i32.to_le_bytes().to_vec();
    out.extend_from_slice(&code.to_le_bytes());
    out
}

/// A child queues a bounded public path; the root executes it after eval.
pub fn dispatch(req: &[u8]) -> Vec<u8> {
    let Some(&op) = req.first() else {
        return invalid();
    };
    if op != OP_PUBLISH && op != OP_UNPUBLISH {
        return invalid();
    }
    let Some(len) = req.get(1..5).and_then(|b| <[u8; 4]>::try_from(b).ok()) else {
        return invalid();
    };
    let len = u32::from_le_bytes(len) as usize;
    let Some(rel) = req.get(5..5usize.saturating_add(len)) else {
        return invalid();
    };
    if 5 + len != req.len() || !super::queue::enqueue(op == OP_PUBLISH, rel) {
        return super::ERR_VALIDATION.to_le_bytes().to_vec();
    }
    accepted(0)
}
