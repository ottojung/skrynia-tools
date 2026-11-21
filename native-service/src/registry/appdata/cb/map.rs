use super::AppErr;
use super::resp::{conflict, err, ok};

pub fn map_read<D, L>(r: Result<Vec<u8>, AppErr<D, L>>) -> Vec<u8> {
    match r {
        Ok(b) => ok(b),
        Err(AppErr::NotFound) => err(8),
        Err(AppErr::BadAuth | AppErr::BadKey) => err(9),
        Err(AppErr::Conflict(_)) => err(10),
        Err(AppErr::Busy) => err(11),
        Err(_) => err(7),
    }
}

pub fn map_write<D, L>(r: Result<[u8; 32], AppErr<D, L>>) -> Vec<u8> {
    match r {
        Ok(h) => ok(h.to_vec()),
        Err(AppErr::Conflict(c)) => conflict(c),
        Err(AppErr::Busy) => err(11),
        Err(AppErr::BadAuth | AppErr::BadKey) => err(9),
        Err(_) => err(7),
    }
}

pub fn map_unit<D, L>(r: Result<(), AppErr<D, L>>) -> Vec<u8> {
    match r {
        Ok(()) => ok(Vec::new()),
        Err(AppErr::Conflict(c)) => conflict(c),
        Err(AppErr::Busy) => err(11),
        Err(AppErr::BadAuth | AppErr::BadKey) => err(9),
        Err(AppErr::NotFound) => conflict(None),
        Err(_) => err(7),
    }
}
