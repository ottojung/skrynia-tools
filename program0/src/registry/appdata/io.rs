use super::cap::Disk;
use super::codec::{decode, encode};
use super::path::data_path;
use super::store_io::StoreErr;

pub fn put<D: Disk>(
    disk: &D,
    base: &str,
    app: &[u8],
    key: &[u8],
    value: &[u8],
) -> Result<(), StoreErr<D::Err>> {
    let path = data_path(base, app, key).ok_or(StoreErr::BadId)?;
    let rec = encode(key, value).ok_or(StoreErr::BadRecord)?;
    disk.write_atomic(&path, &rec).map_err(StoreErr::Disk)
}

pub fn get<D: Disk>(
    disk: &D,
    base: &str,
    app: &[u8],
    key: &[u8],
) -> Result<Vec<u8>, StoreErr<D::Err>> {
    let path = data_path(base, app, key).ok_or(StoreErr::BadId)?;
    let bytes = disk
        .read_all(&path)
        .map_err(StoreErr::Disk)?
        .ok_or(StoreErr::Missing)?;
    decode(&bytes, key).ok_or(StoreErr::BadRecord)
}
