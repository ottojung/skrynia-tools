use super::helpers::{put_blob, put_id};
use crate::wire::{
    OP_PUSH_GRANT, OP_PUSH_SENDER_GRANT, OP_PUSH_SUBSCRIBE, OP_PUSH_UNSUBSCRIBE, Request,
};

pub fn is_push(req: &Request) -> bool {
    matches!(
        req,
        Request::PushSubscribe { .. }
            | Request::PushUnsubscribe { .. }
            | Request::PushGrant { .. }
            | Request::PushSenderGrant { .. }
    )
}

pub fn encode_push(out: &mut Vec<u8>, req: &Request) {
    match req {
        Request::PushSubscribe(grant_cap, key, subscriber, subscription) => {
            out.push(OP_PUSH_SUBSCRIBE);
            put_id(out, grant_cap);
            put_blob(out, key);
            put_blob(out, subscriber);
            put_blob(out, subscription);
        }
        Request::PushUnsubscribe(grant_cap, key, subscriber) => {
            out.push(OP_PUSH_UNSUBSCRIBE);
            put_id(out, grant_cap);
            put_blob(out, key);
            put_blob(out, subscriber);
        }
        Request::PushGrant(write_cap, grant_cap, key, subscriber) => {
            out.push(OP_PUSH_GRANT);
            put_id(out, write_cap);
            put_id(out, grant_cap);
            put_blob(out, key);
            put_blob(out, subscriber);
        }
        Request::PushSenderGrant(write_cap, sender_cap, key) => {
            out.push(OP_PUSH_SENDER_GRANT);
            put_id(out, write_cap);
            put_id(out, sender_cap);
            put_blob(out, key);
        }
        _ => unreachable!(),
    }
}
