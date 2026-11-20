use super::StoreErr;
use crate::AppErr;

#[derive(Debug)]
pub enum RegErr<D, E, R, L> {
    Store(StoreErr<D>),
    App(AppErr<D, L>),
    Eval(E),
    Rand(R),
    Lock(L),
    BadAuth,
    BadId,
    Missing,
    Conflict,
    Busy,
}

impl<D, E, R, L> RegErr<D, E, R, L> {
    pub fn from_app(e: AppErr<D, L>) -> Self {
        match e {
            AppErr::Disk(d) => RegErr::Store(StoreErr::Disk(d)),
            AppErr::Lock(l) => RegErr::Lock(l),
            AppErr::BadAuth | AppErr::BadKey => RegErr::BadAuth,
            AppErr::NotFound => RegErr::Missing,
            AppErr::Busy => RegErr::Busy,
            AppErr::Conflict(_) => RegErr::Conflict,
        }
    }
}
