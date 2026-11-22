use super::helpers::{put_blob, put_id};
use crate::wire::{OP_PUSH_SUBSCRIBE, OP_PUSH_UNSUBSCRIBE, Request};

pub fn encode_push(out: &mut Vec<u8>, req: &Request) {
    match req {
        Request::PushSubscribe(write_cap, key, subscriber, subscription) => {
            out.push(OP_PUSH_SUBSCRIBE);
            put_id(out, &write_cap);
            put_blob(out, &key);
            put_blob(out, &subscriber);
            put_blob(out, &subscription);
        }
        Request::PushUnsubscribe(write_cap, key, subscriber) => {
            out.push(OP_PUSH_UNSUBSCRIBE);
            put_id(out, &write_cap);
            put_blob(out, &key);
            put_blob(out, &subscriber);
        }
        _ => unreachable!(),
    }
}
