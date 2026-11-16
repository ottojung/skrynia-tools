//! Optional immutable-manifest snapshots with single-head atomic CAS.
pub(super) use super::{CreateResult, SkryniaClient, Store, wire};
mod changes;
mod commit;
mod init;
mod keys;
mod native;
mod read;
mod stage;
mod types;
pub use types::{BlobRef, Commit, Head, Manifest, Snapshot, SnapshotBackend, SnapshotStore};
#[cfg(test)]
mod tests;
pub(super) fn digest(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(bytes))
}
