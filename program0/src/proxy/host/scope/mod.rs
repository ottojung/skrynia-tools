mod fd;
pub mod meta;
mod meta_dec;
mod meta_load;
mod path;
mod state;
pub use meta::op;
pub use path::map_path;
pub use state::{enter, exit, with};

pub fn allowed(bit: u32) -> bool {
    with(|c| c.policy.as_ref().is_some_and(|p| p.allows(bit))).unwrap_or(false)
}
pub fn map(p: &[u8]) -> Option<Vec<u8>> {
    with(|c| c.policy.as_ref().and_then(|pol| map_path(pol, p))).flatten()
}
pub fn fd_new(real: i32) -> Option<i32> {
    with(|c| c.fds.insert(real)).flatten()
}
pub fn fd_get(h: i32) -> Option<i32> {
    with(|c| c.fds.get(h)).flatten()
}
pub fn fd_take(h: i32) -> Option<i32> {
    with(|c| c.fds.take(h)).flatten()
}
