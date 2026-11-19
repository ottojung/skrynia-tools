use super::{cap::Disk, paths};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Record {
    pub key: String,
    pub body: Vec<u8>,
}

pub fn read<D: Disk>(disk: &D, key: &str) -> Result<Option<Vec<u8>>, D::Err> {
    let Some(bytes) = disk.read_all(&paths::obj_path(key))? else {
        return Ok(None);
    };
    let rec: Option<Record> = serde_json::from_slice(&bytes).ok();
    Ok(rec.and_then(|r| if r.key == key { Some(r.body) } else { None }))
}

pub fn write<D: Disk>(disk: &D, key: &str, body: &[u8]) -> Result<(), D::Err> {
    let rec = Record {
        key: key.to_string(),
        body: body.to_vec(),
    };
    let bytes = serde_json::to_vec(&rec).unwrap_or_default();
    disk.write_atomic(&paths::obj_path(key), &bytes)
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
