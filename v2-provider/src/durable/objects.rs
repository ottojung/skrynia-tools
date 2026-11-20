use crate::bridge::Callback;
use crate::bridge::sys::{self, fs, path};
pub fn key_path(key: &str) -> String {
    format!("v2/objects/{key}")
}

pub fn read<I: Callback>(i: &I, key: &str) -> Result<Option<Vec<u8>>, ()> {
    sys::io::read_all(i, &key_path(key)).map_err(|_| ())
}
pub fn write<I: Callback>(i: &I, key: &str, body: &[u8]) -> Result<(), ()> {
    fs::write_atomic(i, &key_path(key), body).map_err(|_| ())
}
pub fn remove<I: Callback>(i: &I, key: &str) -> Result<(), ()> {
    let p = key_path(key);
    if path::delete(i, &p).map_err(|_| ())? < 0 {
        return Err(());
    }
    if let Some((dir, _)) = p.rsplit_once('/') {
        fs::sync_dir(i, dir).map_err(|_| ())?;
    }
    Ok(())
}
