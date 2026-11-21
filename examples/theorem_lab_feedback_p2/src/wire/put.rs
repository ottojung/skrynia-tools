use super::{OP_APP_PUT, PRECOND_ABSENT, PRECOND_PRESENT, VERSION, put_blob, put_id};
use alloc::vec::Vec;

pub fn app_put_absent(cap: &[u8], key: &[u8], value: &[u8]) -> Vec<u8> {
    let mut out = alloc::vec![VERSION, OP_APP_PUT];
    put_id(&mut out, cap);
    put_blob(&mut out, key);
    out.push(PRECOND_ABSENT);
    put_blob(&mut out, value);
    out
}
pub fn app_put_present(cap: &[u8], key: &[u8], hash: &[u8; 32], value: &[u8]) -> Vec<u8> {
    let mut out = alloc::vec![VERSION, OP_APP_PUT];
    put_id(&mut out, cap);
    put_blob(&mut out, key);
    out.push(PRECOND_PRESENT);
    out.extend_from_slice(hash);
    put_blob(&mut out, value);
    out
}
