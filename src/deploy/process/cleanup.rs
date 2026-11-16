use super::{run, status};
use anyhow::{Result, bail};
use std::{fs, path::Path, process::Command};

pub fn stale(root: &Path, name: &str) -> Result<()> {
    let prefix = format!("{name}-");
    for entry in fs::read_dir(root.join(".work"))? {
        let entry = entry?;
        let filename = entry.file_name();
        let filename = filename.to_str().unwrap_or("");
        let Some(id) = filename.strip_prefix(&prefix) else {
            continue;
        };
        if id.len() != 32 || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
            continue;
        }
        let container = format!("skrynia-{filename}");
        let mut probe = Command::new("podman");
        probe.args(["container", "exists", &container]);
        let status = status(probe, 20)?;
        match status.code() {
            Some(0) => {
                let mut command = Command::new("podman");
                command.args(["rm", "-f", &container]);
                run(command, 20)?;
            }
            Some(1) => {}
            _ => bail!("could not inspect previous build {container}"),
        }
        if !entry.file_type()?.is_dir() {
            bail!("stale work entry is not a directory");
        }
        fs::remove_dir_all(entry.path())?;
    }
    Ok(())
}
