pub mod app_history;
mod git;
mod run;
mod shell;
use std::path::PathBuf;
pub struct Fixture {
    pub root: PathBuf,
    pub source: PathBuf,
    pub apps: PathBuf,
}
impl Fixture {
    pub fn new() -> Self {
        shell::create()
    }
    pub fn commit(&self, text: &str) -> String {
        git::commit(self, text)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
