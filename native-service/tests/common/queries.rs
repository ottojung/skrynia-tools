//! Disk-state query helpers for wake fan-out tests.
use super::{Mem, caps};
use skrynia_native_service::cap::Disk;
use skrynia_native_service::read_cap;
use skrynia_native_service::registry::appdata::{self as ad, push};

pub fn hints(d: &Mem, key: &[u8]) -> usize {
    d.list(&push::hints_root("d", &read_cap(&caps().0), key))
        .unwrap()
        .len()
}
pub fn data_exists(d: &Mem, key: &[u8]) -> bool {
    d.read_all(&ad::path::data_path("d", &read_cap(&caps().0), key).unwrap())
        .unwrap()
        .is_some()
}
