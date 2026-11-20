use crate::wire::{AppPrecond, PRECOND_ABSENT, PRECOND_PRESENT};

pub fn precond(src: &mut &[u8]) -> Result<AppPrecond, super::super::take::DecodeErr> {
    match super::super::take::take(src, 1)?[0] {
        PRECOND_ABSENT => Ok(AppPrecond::Absent),
        PRECOND_PRESENT => Ok(AppPrecond::Present(hash(src)?)),
        _ => Err(super::super::take::DecodeErr::Op),
    }
}

pub fn hash(src: &mut &[u8]) -> Result<Vec<u8>, super::super::take::DecodeErr> {
    Ok(super::super::take::take(src, 32)?.to_vec())
}
