mod bootstrap;
mod provision;
mod secret;
use super::{auth::Authority, cap, cap::Disk, lock, paths};
pub use bootstrap::bootstrap_root;
pub use provision::ensure_root;
#[derive(Debug)]
pub enum StateErr<E> {
    Disk(E),
    Decode,
    Uncertain,
    Exists,
    Missing,
}
pub fn is_uncertain<D: Disk>(disk: &D) -> Result<bool, D::Err> {
    Ok(disk.read_all(paths::UNCERTAIN)?.is_some())
}
pub fn mark_uncertain<D: Disk>(disk: &D) -> Result<(), D::Err> {
    disk.write_atomic(paths::UNCERTAIN, b"1")
}
pub fn clear_uncertain<D: Disk>(disk: &D) -> Result<(), D::Err> {
    disk.remove(paths::UNCERTAIN)
}
pub fn load<D: Disk>(disk: &D) -> Result<Authority, StateErr<D::Err>> {
    if is_uncertain(disk).map_err(StateErr::Disk)? {
        return Err(StateErr::Uncertain);
    }
    let bytes = disk
        .read_all(paths::AUTH)
        .map_err(StateErr::Disk)?
        .ok_or(StateErr::Missing)?;
    Authority::decode(&bytes).ok_or(StateErr::Decode)
}
