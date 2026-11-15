use super::{Release, files};
use std::{fs, path::PathBuf};
pub struct Fixture {
    pub(super) root: PathBuf,
}
impl Fixture {
    pub fn new() -> Self {
        let root =
            std::env::temp_dir().join(format!("skrynia-release-{:032x}", rand::random::<u128>()));
        files::ensure_dirs(&root).unwrap();
        Self { root }
    }
    pub fn add(&self, n: u64) -> Release {
        let id = format!("{n:012x}-{n:016x}");
        let name = "hello";
        let tree = self.root.join("releases").join(name).join(&id);
        fs::create_dir_all(&tree).unwrap();
        fs::write(tree.join("index.html"), n.to_string()).unwrap();
        let entry = Release {
            id: id.clone(),
            name: name.into(),
            repo: "https://github.com/ottojung/demo".into(),
            commit: format!("{n:012x}{}", "a".repeat(28)),
            subdir: ".".into(),
            builder_image: "builder".into(),
            published_unix_ms: Some(n),
            content_sha256: None,
        };
        let dir = self.root.join("metadata").join(name);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(format!("{id}.json")),
            serde_json::to_vec(&entry).unwrap(),
        )
        .unwrap();
        entry
    }
    pub fn active(&self, id: &str) {
        files::switch(&self.root, "hello", id).unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
