use super::io;
use super::valid_key;
use super::{AppErr, CAP_LEN, Disk, State};

pub fn get<D: Disk>(
    disk: &D,
    base: &str,
    read_cap: &[u8],
    key: &[u8],
) -> Result<Vec<u8>, AppErr<D::Err, ()>> {
    if read_cap.len() != CAP_LEN || !valid_key(key) {
        return Err(AppErr::BadKey);
    }
    match io::read_state(disk, base, read_cap, key).map_err(AppErr::from_store)? {
        State::Absent => Err(AppErr::NotFound),
        State::Present(v) => Ok(v),
    }
}
