use super::cap::Disk;
use super::path::data_path;
use super::store_io::StoreErr;
use crate::hash_bytes;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    Absent,
    Present(Vec<u8>),
}

impl State {
    pub fn hash(&self) -> Option<[u8; 32]> {
        match self {
            State::Absent => None,
            State::Present(v) => Some(hash_bytes(v)),
        }
    }
}

pub fn read_state<D: Disk>(
    disk: &D,
    base: &str,
    read_cap: &[u8],
    key: &[u8],
) -> Result<State, StoreErr<D::Err>> {
    let path = data_path(base, read_cap, key).ok_or(StoreErr::BadId)?;
    match disk.read_all(&path).map_err(StoreErr::Disk)? {
        None => Ok(State::Absent),
        Some(b) => Ok(State::Present(b)),
    }
}

pub fn write_value<D: Disk>(
    disk: &D,
    base: &str,
    read_cap: &[u8],
    key: &[u8],
    value: &[u8],
) -> Result<[u8; 32], StoreErr<D::Err>> {
    let path = data_path(base, read_cap, key).ok_or(StoreErr::BadId)?;
    disk.write_atomic(&path, value).map_err(StoreErr::Disk)?;
    Ok(hash_bytes(value))
}

pub fn delete_value<D: Disk>(
    disk: &D,
    base: &str,
    read_cap: &[u8],
    key: &[u8],
) -> Result<(), StoreErr<D::Err>> {
    let path = data_path(base, read_cap, key).ok_or(StoreErr::BadId)?;
    disk.remove(&path).map_err(StoreErr::Disk)
}
