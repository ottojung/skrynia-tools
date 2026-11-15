use super::{process, types::DeploySpec};
use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub fn build(spec: &DeploySpec, work: &Path) -> Result<()> {
    let source = work.join("source").canonicalize()?;
    let user = unsafe { libc::geteuid() };
    let group = unsafe { libc::getegid() };
    let name = format!("skrynia-build-{:016x}", rand::random::<u64>());
    let mut cmd = Command::new("podman");
    cmd.args([
        "run",
        "--rm",
        "--name",
        &name,
        "--network",
        "bridge",
        "--cap-drop",
        "ALL",
        "--security-opt",
        "no-new-privileges",
        "--user",
        &format!("{user}:{group}"),
        "--env",
        "HOME=/tmp",
        "-v",
        &format!("{}:/workspace:Z", source.display()),
        "-w",
        &format!("/workspace/{}", spec.subdir),
        &spec.builder_image,
        "make",
        "build",
    ]);
    let result = process::run(cmd, 1800);
    if result.is_err() {
        let _ = Command::new("podman").args(["rm", "-f", &name]).output();
    }
    result
}
