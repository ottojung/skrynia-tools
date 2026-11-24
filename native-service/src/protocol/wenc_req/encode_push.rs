use super::helpers::{put_blob, put_id};
use crate::wire::*;
pub fn is_push(req: &Request) -> bool {
    matches!(
        req,
        Request::PushSubscribe(..)
            | Request::PushUnsubscribe(..)
            | Request::PushGrant(..)
            | Request::PushSenderGrant(..)
            | Request::PushWakeGrant(..)
    )
}
pub fn encode_push(out: &mut Vec<u8>, req: &Request) {
    match req {
        Request::PushSubscribe(g, k, s, sub) => {
            out.push(OP_PUSH_SUBSCRIBE);
            put_id(out, g);
            put_blob(out, k);
            put_blob(out, s);
            put_blob(out, sub);
        }
        Request::PushUnsubscribe(g, k, s) => {
            out.push(OP_PUSH_UNSUBSCRIBE);
            put_id(out, g);
            put_blob(out, k);
            put_blob(out, s);
        }
        Request::PushGrant(w, g, k, s) => {
            out.push(OP_PUSH_GRANT);
            put_id(out, w);
            put_id(out, g);
            put_blob(out, k);
            put_blob(out, s);
        }
        Request::PushSenderGrant(w, g, k) => {
            out.push(OP_PUSH_SENDER_GRANT);
            put_id(out, w);
            put_id(out, g);
            put_blob(out, k);
        }
        Request::PushWakeGrant(w, g, k, ns) => {
            out.push(OP_PUSH_WAKE_GRANT);
            put_id(out, w);
            put_id(out, g);
            put_blob(out, k);
            put_id(out, ns);
        }
        _ => unreachable!(),
    }
}
