mod apply;
mod auth;
mod lock;
mod mutate;
mod object;
mod paths;
pub mod state;
mod wire;
mod wmut;
#[allow(unused_imports)]
pub(crate) use super::host;
pub(crate) use super::{cap, protocol};
pub use mutate::{MutateErr, MutateKind, MutateReq};
pub use state::{StateErr, bootstrap_root, ensure_root};
pub use wire::handle_wire;
pub fn mutate<D: cap::Disk>(disk: &D, req: MutateReq<'_>) -> Result<bool, MutateErr<D::Err>> {
    apply::run(disk, req)
}
