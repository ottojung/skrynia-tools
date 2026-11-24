//! Apply the resolved key to the publication root. Repeated operations are
//! idempotent only when the existing node is the expected symlink or absent.

use super::abi;
use super::change;
use super::dest;
use super::link;
use std::os::fd::OwnedFd;
use std::path::Path;

pub(super) fn publish_to(root: &OwnedFd, parts: &[String], target: &Path) -> u32 {
    let (dir, name) = match dest::parent(root, parts, true) {
        Ok(Some(found)) => found,
        Ok(None) => return abi::INTERNAL,
        Err(code) => return code,
    };
    match link::inspect(&dir, &name, target) {
        Ok(link::Found::Same) => abi::OK,
        Ok(link::Found::None) => change::create(&dir, &name, target),
        Ok(link::Found::Other) => abi::DEST_CONFLICT,
        Err(code) => code,
    }
}

pub(super) fn unpublish_from(root: &OwnedFd, parts: &[String], target: &Path) -> u32 {
    let (dir, name) = match dest::parent(root, parts, false) {
        Ok(Some(found)) => found,
        Ok(None) => return abi::OK,
        Err(code) => return code,
    };
    match link::inspect(&dir, &name, target) {
        Ok(link::Found::Same) => change::remove(&dir, &name),
        Ok(link::Found::None) => abi::OK,
        Ok(link::Found::Other) => abi::DEST_CONFLICT,
        Err(code) => code,
    }
}
