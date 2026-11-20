use super::Server;
impl Server {
    pub fn directory_names(&self, key: &str) -> Vec<String> {
        let path = self.dir.join(key);
        std::fs::read_dir(path)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect()
    }
}
