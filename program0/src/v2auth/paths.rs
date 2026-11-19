use super::cap::Disk;
use sha2::{Digest, Sha256};
pub const DIR: &str = "v2auth";
pub const AUTH: &str = "v2auth/authority.json";
pub const UNCERTAIN: &str = "v2auth/uncertain";
pub const OBJS: &str = "v2auth/objs";

pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::new().chain_update(bytes).finalize().into()
}
pub fn hex32(bytes: &[u8; 32]) -> String {
    hex::encode(bytes)
}
pub fn ensure<D: Disk>(disk: &D) -> Result<(), D::Err> {
    mkdir(disk, DIR)?;
    mkdir(disk, OBJS)
}
fn mkdir<D: Disk>(disk: &D, dir: &str) -> Result<(), D::Err> {
    #[cfg(target_arch = "wasm32")]
    {
        let rc = unsafe { super::host::root_abi::mkdir(dir.as_ptr() as u32, dir.len() as u32, 4) };
        if rc < 0 {
            return Err(rc);
        }
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (disk, dir);
        Ok(())
    }
}
pub fn obj_path(key: &str) -> String {
    format!("{OBJS}/{}", hex32(&sha256(key.as_bytes())))
}
