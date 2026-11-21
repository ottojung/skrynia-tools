use crate::CAP_LEN;
use crate::read_cap_vec;

#[derive(Debug, Clone)]
pub enum AppErr<D, L> {
    Disk(D),
    Lock(L),
    BadAuth,
    BadKey,
    NotFound,
    Busy,
    Conflict(Option<[u8; 32]>),
}

impl<D, L> AppErr<D, L> {
    pub(crate) fn from_store(e: super::StoreErr<D>) -> Self {
        match e {
            super::StoreErr::Disk(d) => AppErr::Disk(d),
            super::StoreErr::Missing => AppErr::NotFound,
            _ => AppErr::BadKey,
        }
    }
}

pub enum WriteAuth<'a> {
    External(&'a [u8]),
    SelfNs(&'a [u8]),
}

impl<'a> WriteAuth<'a> {
    pub(crate) fn namespace(&self) -> Result<Vec<u8>, ()> {
        match self {
            WriteAuth::External(w) => {
                if w.len() != CAP_LEN {
                    return Err(());
                }
                Ok(read_cap_vec(w))
            }
            WriteAuth::SelfNs(ns) => {
                if ns.len() != CAP_LEN {
                    return Err(());
                }
                Ok(ns.to_vec())
            }
        }
    }
}
