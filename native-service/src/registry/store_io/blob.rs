use super::{Disk, StoreErr};
use crate::store;

pub fn ensure_blob<D: Disk>(
    disk: &D,
    dir: &str,
    code: &[u8],
) -> Result<[u8; 32], StoreErr<D::Err>> {
    let hash = store::code_hash(code);
    let path = store::blob_path(dir, &hash).ok_or(StoreErr::BadId)?;
    let encoded = store::encode_blob(code);
    match disk.read_all(&path).map_err(StoreErr::Disk)? {
        None => disk.write_atomic(&path, &encoded).map_err(StoreErr::Disk)?,
        Some(existing) if existing == encoded => {}
        Some(_) => return Err(StoreErr::BadRecord),
    }
    Ok(hash)
}

pub fn read_blob<D: Disk>(
    disk: &D,
    dir: &str,
    code_hash: &[u8; 32],
) -> Result<Vec<u8>, StoreErr<D::Err>> {
    let path = store::blob_path(dir, code_hash).ok_or(StoreErr::BadId)?;
    let bytes = disk
        .read_all(&path)
        .map_err(StoreErr::Disk)?
        .ok_or(StoreErr::Missing)?;
    let code = store::decode_blob(&bytes).ok_or(StoreErr::BadRecord)?;
    if store::code_hash(&code) != *code_hash {
        return Err(StoreErr::BadRecord);
    }
    Ok(code)
}
