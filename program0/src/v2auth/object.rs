use super::{MutateErr, cap::Disk, paths};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Record {
    pub key: String,
    pub body: Vec<u8>,
}

pub fn read<D: Disk>(disk: &D, key: &str) -> Result<Option<Vec<u8>>, MutateErr<D::Err>> {
    let Some(bytes) = disk
        .read_all(&paths::obj_path(key))
        .map_err(MutateErr::Disk)?
    else {
        return Ok(None);
    };
    let rec: Record = serde_json::from_slice(&bytes).map_err(|_| MutateErr::Decode)?;
    if rec.key != key {
        return Err(MutateErr::Decode);
    }
    Ok(Some(rec.body))
}

pub fn write<D: Disk>(disk: &D, key: &str, body: &[u8]) -> Result<(), MutateErr<D::Err>> {
    let rec = Record {
        key: key.to_string(),
        body: body.to_vec(),
    };
    let bytes = serde_json::to_vec(&rec).map_err(|_| MutateErr::Decode)?;
    disk.write_atomic(&paths::obj_path(key), &bytes)
        .map_err(MutateErr::Disk)
}

pub fn remove<D: Disk>(disk: &D, key: &str) -> Result<(), D::Err> {
    disk.remove(&paths::obj_path(key))
}

pub fn cas_ok(current: Option<&[u8]>, expected_hex: Option<&[u8; 32]>) -> bool {
    let Some(exp) = expected_hex else {
        return true;
    };
    match current {
        Some(bytes) => paths::sha256(bytes) == *exp,
        None => false,
    }
}
