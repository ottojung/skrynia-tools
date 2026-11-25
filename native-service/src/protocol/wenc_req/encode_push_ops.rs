//! Body encoders for ordinary P1 push opcodes.
use super::helpers::{put_blob, put_id};
use crate::wire::*;

pub fn subscribe(out: &mut Vec<u8>, g: &[u8], k: &[u8], s: &[u8], sub: &[u8]) {
    out.push(OP_PUSH_SUBSCRIBE);
    put_id(out, g);
    put_blob(out, k);
    put_blob(out, s);
    put_blob(out, sub);
}

pub fn unsubscribe(out: &mut Vec<u8>, g: &[u8], k: &[u8], s: &[u8]) {
    out.push(OP_PUSH_UNSUBSCRIBE);
    put_id(out, g);
    put_blob(out, k);
    put_blob(out, s);
}

pub fn grant(out: &mut Vec<u8>, w: &[u8], g: &[u8], k: &[u8], s: &[u8]) {
    out.push(OP_PUSH_GRANT);
    put_id(out, w);
    put_id(out, g);
    put_blob(out, k);
    put_blob(out, s);
}

pub fn sender_grant(out: &mut Vec<u8>, w: &[u8], g: &[u8], k: &[u8]) {
    out.push(OP_PUSH_SENDER_GRANT);
    put_id(out, w);
    put_id(out, g);
    put_blob(out, k);
}

pub fn wake_grant(out: &mut Vec<u8>, w: &[u8], g: &[u8], k: &[u8], ns: &[u8]) {
    out.push(OP_PUSH_WAKE_GRANT);
    put_id(out, w);
    put_id(out, g);
    put_blob(out, k);
    put_id(out, ns);
}

pub fn sub_status(out: &mut Vec<u8>, g: &[u8], k: &[u8], s: &[u8], h: &[u8]) {
    out.push(OP_SUB_STATUS);
    put_id(out, g);
    put_blob(out, k);
    put_blob(out, s);
    put_id(out, h);
}
