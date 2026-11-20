use super::helpers::{put_blob, put_hash, put_id, put_precond};
use crate::wire::{OP_APP_DELETE, OP_APP_GET, OP_APP_PUT, Request};

pub fn encode_app(out: &mut Vec<u8>, req: &Request) {
    match req {
        Request::AppGet { read_cap, key } => {
            out.push(OP_APP_GET);
            put_id(out, read_cap);
            put_blob(out, key);
        }
        Request::AppPut {
            write_cap,
            key,
            expect,
            value,
        } => {
            out.push(OP_APP_PUT);
            put_id(out, write_cap);
            put_blob(out, key);
            put_precond(out, expect);
            put_blob(out, value);
        }
        Request::AppDelete {
            write_cap,
            key,
            expect,
        } => {
            out.push(OP_APP_DELETE);
            put_id(out, write_cap);
            put_blob(out, key);
            put_hash(out, expect);
        }
        _ => unreachable!(),
    }
}
