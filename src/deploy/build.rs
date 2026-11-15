use super::{process, types::DeploySpec};
use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub fn build(spec: &DeploySpec, work: &Path) -> Result<()> {
    let source = work.join("source").canonicalize()?;
    let mut info = Command::new("podman");
    info.args(["info", "--format", "{{.Host.Security.Rootless}}"]);
    if process::capture(info, 15)? != b"true\n" {
        anyhow::bail!("app builds require rootless Podman");
    }
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
        "0:0",
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
        let mut cleanup = Command::new("podman");
        cleanup.args(["rm", "-f", &name]);
        let _ = process::run(cleanup, 20);
    }
    result
}
