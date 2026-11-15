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
    pub fn break_build(&self) {
        git::break_build(self);
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
