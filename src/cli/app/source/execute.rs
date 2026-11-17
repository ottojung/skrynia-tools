use super::Arguments;
use anyhow::{Result, bail};
use std::{
    path::{Component, Path, PathBuf},
    process::Command,
};
pub fn run(args: &Arguments, root: &Path) -> Result<PathBuf> {
    let subdir = Path::new(args.optional("subdir").unwrap_or("."));
    let output = Path::new(args.optional("output").unwrap_or("build"));
    for path in [subdir, output] {
        if !path
            .components()
            .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
        {
            bail!("build path escapes the repository");
        }
    }
    let source = root.join(subdir);
    if source.is_symlink() || !source.join("Makefile").is_file() {
        bail!("application subdirectory has no regular Makefile");
    }
    let token_env = args.optional("token-env").unwrap_or("SKRYNIA_TOKEN");
    let result = Command::new("make")
        .arg("build")
        .current_dir(&source)
        .env_remove("SKRYNIA_TOKEN")
        .env_remove(token_env)
        .output()?;
    eprint!("{}", String::from_utf8_lossy(&result.stdout));
    eprint!("{}", String::from_utf8_lossy(&result.stderr));
    if !result.status.success() {
        bail!("local source build failed");
    }
    let directory = source.join(output);
    if !directory.join("index.html").is_file() {
        bail!("build has no index.html");
    }
    Ok(directory)
}
