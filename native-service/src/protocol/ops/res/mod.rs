mod err;
use crate::reg::RegErr;
pub use err::{conflict, err as mk_err, ok};

pub fn map_res<D, E, R, L>(r: Result<Vec<u8>, RegErr<D, E, R, L>>, default: u8) -> Vec<u8> {
    match r {
        Ok(v) => v,
        Err(RegErr::BadAuth | RegErr::BadId) => mk_err(9),
        Err(RegErr::Missing) => mk_err(8),
        Err(RegErr::Conflict) => conflict(None),
        Err(RegErr::Busy) => mk_err(11),
        Err(_) => mk_err(default),
    }
}

pub fn map_app_get<D, L>(r: Result<Vec<u8>, RegErr<D, (), (), L>>) -> Vec<u8> {
    match r {
        Ok(v) => ok(v),
        Err(RegErr::Missing) => mk_err(8),
        Err(RegErr::BadAuth | RegErr::BadId) => mk_err(9),
        Err(RegErr::Busy) => mk_err(11),
        Err(_) => mk_err(7),
    }
}

pub fn map_app_write<D, L>(r: Result<Vec<u8>, RegErr<D, (), (), L>>) -> Vec<u8> {
    match r {
        Ok(v) => ok(v),
        Err(RegErr::Conflict) => conflict(None),
        Err(RegErr::Busy) => mk_err(11),
        Err(RegErr::BadAuth | RegErr::BadId) => mk_err(9),
        Err(_) => mk_err(7),
    }
}

pub fn map_app_unit<D, L>(r: Result<(), RegErr<D, (), (), L>>) -> Vec<u8> {
    match r {
        Ok(()) => ok(Vec::new()),
        Err(RegErr::Conflict) => conflict(None),
        Err(RegErr::Busy) => mk_err(11),
        Err(RegErr::BadAuth | RegErr::BadId) => mk_err(9),
        Err(RegErr::Missing) => mk_err(8),
        Err(_) => mk_err(7),
    }
}
