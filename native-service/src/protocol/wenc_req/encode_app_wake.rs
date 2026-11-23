use super::helpers::{put_blob, put_hash, put_id, put_precond};
use crate::wire::{OP_APP_DELETE_WAKE, OP_APP_PUT_WAKE, Request};
pub fn encode_app_wake(out: &mut Vec<u8>, req: &Request) {
    match req {
        Request::AppPutWake(w, k, e, v, g, wk) => {
            out.push(OP_APP_PUT_WAKE);
            put_id(out, w);
            put_blob(out, k);
            put_precond(out, e);
            put_blob(out, v);
            put_id(out, g);
            put_blob(out, wk);
        }
        Request::AppDeleteWake(w, k, e, g, wk) => {
            out.push(OP_APP_DELETE_WAKE);
            put_id(out, w);
            put_blob(out, k);
            put_hash(out, e);
            put_id(out, g);
            put_blob(out, wk);
        }
        _ => unreachable!(),
    }
}
