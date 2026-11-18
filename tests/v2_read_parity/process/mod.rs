mod launch;
mod lifecycle;
use std::{path::PathBuf, process::Child};
pub struct Server {
    pub url: String,
    pub(super) child: Child,
    pub(super) dir: PathBuf,
}
