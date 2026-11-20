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
