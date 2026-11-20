use crate::wire::{AppPrecond, PRECOND_ABSENT, PRECOND_PRESENT};

pub fn put_precond(out: &mut Vec<u8>, p: &AppPrecond) {
    match p {
        AppPrecond::Absent => out.push(PRECOND_ABSENT),
        AppPrecond::Present(h) => {
            out.push(PRECOND_PRESENT);
            put_hash(out, h);
        }
    }
}

pub fn put_hash(out: &mut Vec<u8>, h: &[u8]) {
    out.extend_from_slice(h);
}

pub fn put_id(out: &mut Vec<u8>, id: &[u8]) {
    out.extend_from_slice(&(id.len() as u16).to_le_bytes());
    out.extend_from_slice(id);
}

pub fn put_blob(out: &mut Vec<u8>, data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(data);
}
