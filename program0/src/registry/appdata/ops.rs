use super::cap::Disk;
use super::id;
use super::io;
use super::path::app_dir;
use super::store_io::StoreErr;

pub fn ensure_dir<D: Disk>(disk: &D, base: &str, app: &[u8]) -> Result<(), StoreErr<D::Err>> {
    let dir = app_dir(base, app).ok_or(StoreErr::BadId)?;
    let _ = disk.list(&dir);
    Ok(())
}

pub fn put<D: Disk>(
    disk: &D,
    base: &str,
    app: &[u8],
    key: &[u8],
    value: &[u8],
) -> Result<(), StoreErr<D::Err>> {
    id::validate(app).map_err(|_| StoreErr::BadId)?;
    ensure_dir(disk, base, app)?;
    io::put(disk, base, app, key, value)
}

pub fn get<D: Disk>(
    disk: &D,
    base: &str,
    app: &[u8],
    key: &[u8],
) -> Result<Vec<u8>, StoreErr<D::Err>> {
    id::validate(app).map_err(|_| StoreErr::BadId)?;
    io::get(disk, base, app, key)
}
