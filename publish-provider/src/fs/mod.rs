//! Publish and unpublish a relative key. Operations are serialized and all
//! filesystem traversal stays inside pre-opened root descriptors.

mod apply;
mod change;
mod dest;
mod link;
mod source;

use super::abi;
use super::mailbox;
use super::path;
use super::roots::Roots;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct State {
    pub roots: Roots,
    pub lock: Mutex<()>,
}

pub(super) fn dup(root: &OwnedFd) -> Result<OwnedFd, u32> {
    let raw = unsafe { libc::dup(root.as_raw_fd()) };
    if raw < 0 {
        Err(abi::IO_ERROR)
    } else {
        Ok(unsafe { OwnedFd::from_raw_fd(raw) })
    }
}

pub fn call(state: &State, ticket: u32, publish: bool) -> u32 {
    let bytes = match mailbox::read(&state.roots.requests, ticket) {
        Ok(bytes) => bytes,
        Err(code) => return code,
    };
    let parts = match path::clean(&bytes) {
        Ok(parts) => parts,
        Err(_) => return abi::INVALID_PATH,
    };
    let target: PathBuf = state.roots.disk_abs.join(parts.iter().collect::<PathBuf>());
    let _guard = match state.lock.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    if publish {
        let _source = match source::open(&state.roots, &parts) {
            Ok(source) => source,
            Err(code) => return code,
        };
        apply::publish_to(&state.roots.nginx, &parts, &target)
    } else {
        apply::unpublish_from(&state.roots.nginx, &parts, &target)
    }
}
