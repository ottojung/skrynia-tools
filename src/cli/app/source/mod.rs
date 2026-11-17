mod checkout;
mod execute;
use super::Arguments;
use anyhow::Result;
use std::{fs, path::PathBuf};
pub struct Build {
    pub directory: PathBuf,
    temporary: Option<PathBuf>,
}
impl Drop for Build {
    fn drop(&mut self) {
        if let Some(root) = &self.temporary {
            let _ = fs::remove_dir_all(root);
        }
    }
}
pub fn prepare(args: &Arguments) -> Result<Build> {
    if let Some(directory) = args.optional("directory") {
        return Ok(Build {
            directory: directory.into(),
            temporary: None,
        });
    }
    let root = checkout::clone(args)?;
    let result = execute::run(args, &root);
    match result {
        Ok(directory) => Ok(Build {
            directory,
            temporary: Some(root),
        }),
        Err(error) => {
            let _ = fs::remove_dir_all(root);
            Err(error)
        }
    }
}
