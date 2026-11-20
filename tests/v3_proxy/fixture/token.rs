use super::Server;
use sha2::{Digest, Sha256};

impl Server {
    pub fn rotate(&self, token: &[u8; 32]) {
        let path = self.dir.join("security/program0.sha256");
        let replacement = path.with_extension("sha256.new");
        std::fs::write(&replacement, Sha256::digest(token)).unwrap();
        std::fs::rename(replacement, path).unwrap();
    }
    pub fn invalidate(&self) {
        std::fs::remove_file(self.dir.join("security/program0.sha256")).unwrap();
    }
    pub fn corrupt(&self, bytes: &[u8]) {
        std::fs::write(self.dir.join("security/program0.sha256"), bytes).unwrap();
    }
}

impl Server {
    pub fn seed_file(&self, key: &str, bytes: &[u8]) {
        let path = self.dir.join(key);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    }
}

impl Server {
    pub fn read_file(&self, key: &str) -> Option<Vec<u8>> {
        std::fs::read(self.dir.join(key)).ok()
    }
}

impl Server {
    pub fn dir_exists(&self, key: &str) -> bool {
        self.dir.join(key).is_dir()
    }
}
#[cfg(unix)]
impl Server {
    pub fn set_file_mode(&self, key: &str, mode: u32) {
        use std::os::unix::fs::PermissionsExt;
        let path = self.dir.join(key);
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
    }
}
