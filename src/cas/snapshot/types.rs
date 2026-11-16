use super::Store;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub trait SnapshotBackend: Store {
    fn compare_head(&self, key: &str, bytes: &[u8], digest: &str) -> Result<bool>;
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Head {
    pub version: u8,
    pub revision: u64,
    pub txid: String,
    pub manifest_sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BlobRef {
    pub key: String,
    pub sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub version: u8,
    pub items: BTreeMap<String, BlobRef>,
}
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub(super) revision: u64,
    pub(super) values: BTreeMap<String, Vec<u8>>,
    pub(super) prefix: String,
    pub(super) refs: BTreeMap<String, BlobRef>,
    pub(super) head_sha256: String,
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Commit {
    Committed,
    Conflict,
}
pub struct SnapshotStore<'a, S: SnapshotBackend> {
    pub(super) store: &'a S,
    pub(super) prefix: String,
}
