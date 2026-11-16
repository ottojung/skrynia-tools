use super::{files, inspect, process, release, valid_name, validate};
mod audit;
use anyhow::Result;
pub use audit::{Audit, audit};
use serde::Serialize;
use std::{path::Path, process::Command};

#[derive(Debug, Serialize)]
pub struct Doctor {
    pub root: String,
    pub root_exists: bool,
    pub git_available: bool,
    pub podman_rootless: bool,
    pub missing_directories: Vec<String>,
    pub prerequisites_ok: bool,
}
pub fn doctor(root: &Path) -> Result<Doctor> {
    let root_exists = files::existing_dir(root)?;
    let git_available = {
        let mut command = Command::new("git");
        command.arg("--version");
        process::capture(command, 15).is_ok()
    };
    let podman_rootless = {
        let mut command = Command::new("podman");
        command.args(["info", "--format", "{{.Host.Security.Rootless}}"]);
        process::capture(command, 15).is_ok_and(|output| output == b"true\n")
    };
    let mut missing_directories = Vec::new();
    for name in ["apps", "releases", "metadata", ".work", ".locks"] {
        if !files::existing_dir(&root.join(name))? {
            missing_directories.push(name.to_owned());
        }
    }
    Ok(Doctor {
        root: root.display().to_string(),
        root_exists,
        git_available,
        podman_rootless,
        missing_directories,
        prerequisites_ok: root_exists && git_available && podman_rootless,
    })
}
