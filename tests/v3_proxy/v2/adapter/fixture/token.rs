use std::{
    fs,
    path::{Path, PathBuf},
};
pub fn create(dir: &Path) -> PathBuf {
    let path = dir.join("operator.token");
    fs::write(&path, [0x42u8; 32]).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    }
    path
}
