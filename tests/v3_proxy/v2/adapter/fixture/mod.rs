mod cleanup;
mod launch;
mod token;
use std::{path::PathBuf, process::Child};
pub struct Gateway {
    pub url: String,
    pub token_path: PathBuf,
    child: Child,
    dir: PathBuf,
}
