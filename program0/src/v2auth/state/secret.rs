use super::cap;
use super::{Disk, StateErr, paths};
pub fn root<D: Disk, R: cap::Random<Err = D::Err>>(
    disk: &D,
    random: &R,
    authority_exists: bool,
) -> Result<[u8; 32], StateErr<D::Err>> {
    if let Some(raw) = disk.read_all(paths::ROOT_SECRET).map_err(StateErr::Disk)? {
        return raw.try_into().map_err(|_| StateErr::Decode);
    }
    if authority_exists {
        return Err(StateErr::Uncertain);
    }
    let mut bytes = [0u8; 32];
    random.fill(&mut bytes).map_err(StateErr::Disk)?;
    disk.write_atomic(paths::ROOT_SECRET, &bytes)
        .map_err(StateErr::Disk)?;
    Ok(bytes)
}
