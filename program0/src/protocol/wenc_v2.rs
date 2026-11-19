use crate::wire::{OP_V2_BOOTSTRAP, OP_V2_MUTATE, Request, V2Kind};

pub fn push(out: &mut Vec<u8>, req: &Request) {
    match req {
        Request::V2Bootstrap { token } => {
            out.push(OP_V2_BOOTSTRAP);
            out.extend_from_slice(token);
        }
        Request::V2Mutate {
            token,
            key,
            kind,
            expected,
            body,
        } => {
            out.push(OP_V2_MUTATE);
            out.extend_from_slice(token);
            out.extend_from_slice(&(key.len() as u16).to_le_bytes());
            out.extend_from_slice(key);
            out.push(match kind {
                V2Kind::Create => 1,
                V2Kind::Replace => 2,
                V2Kind::Delete => 3,
            });
            let flags = if expected.is_some() { 1 } else { 0 };
            out.push(flags);
            if let Some(exp) = expected {
                out.extend_from_slice(exp);
            }
            out.extend_from_slice(&(body.len() as u32).to_le_bytes());
            out.extend_from_slice(body);
        }
        _ => {}
    }
}
