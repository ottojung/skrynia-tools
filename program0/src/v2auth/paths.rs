use super::cap::Disk;
use sha2::{Digest, Sha256};
pub const DIR: &str = "v2auth";
pub const AUTH: &str = "v2auth/authority.json";
pub const ROOT_SECRET: &str = "v2auth/root-secret";
pub const UNCERTAIN: &str = "v2auth/uncertain";
pub const OBJS: &str = "v2auth/objs";

pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::new().chain_update(bytes).finalize().into()
}
pub fn hex32(bytes: &[u8; 32]) -> String {
    hex::encode(bytes)
}
pub fn ensure<D: Disk>(disk: &D) -> Result<(), D::Err> {
    disk.ensure_directory(DIR)?;
    disk.ensure_directory(OBJS)
}
pub fn obj_path(key: &str) -> String {
    format!("{OBJS}/{}", hex32(&sha256(key.as_bytes())))
}
