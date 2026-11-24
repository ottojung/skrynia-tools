use super::helpers::{put_blob, put_id, put_u64};
use crate::wire::{OP_HINT_ACK, OP_HINT_LIST, OP_HINT_READ, OP_PUSH_SUB_FETCH, Request};

pub fn is_sender(req: &Request) -> bool {
    matches!(
        req,
        Request::HintList { .. }
            | Request::HintRead { .. }
            | Request::HintAck { .. }
            | Request::SubFetch { .. }
    )
}

pub fn encode_sender(out: &mut Vec<u8>, req: &Request) {
    match req {
        Request::HintList(cap, ns, key) => {
            out.push(OP_HINT_LIST);
            put_id(out, cap);
            put_id(out, ns);
            put_blob(out, key);
        }
        Request::HintRead(cap, ns, key, seq) => {
            out.push(OP_HINT_READ);
            put_id(out, cap);
            put_id(out, ns);
            put_blob(out, key);
            put_u64(out, *seq);
        }
        Request::HintAck(cap, ns, key, seq) => {
            out.push(OP_HINT_ACK);
            put_id(out, cap);
            put_id(out, ns);
            put_blob(out, key);
            put_u64(out, *seq);
        }
        Request::SubFetch(cap, ns, key) => {
            out.push(OP_PUSH_SUB_FETCH);
            put_id(out, cap);
            put_id(out, ns);
            put_blob(out, key);
        }
        _ => unreachable!(),
    }
}
