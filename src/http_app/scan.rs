use anyhow::{Result, bail};
use std::{
    fs,
    path::{Path, PathBuf},
};
pub fn scan(root: &Path) -> Result<Vec<(String, PathBuf)>> {
    if !root.is_dir() || root.is_symlink() {
        bail!("build directory is not a real directory");
    }
    let mut files = Vec::new();
    visit(root, root, &mut files)?;
    files.sort_by(|a, b| a.0.cmp(&b.0));
    if !files.iter().any(|(path, _)| path == "index.html") {
        bail!("build is missing index.html");
    }
    Ok(files)
}
fn visit(root: &Path, dir: &Path, out: &mut Vec<(String, PathBuf)>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let kind = entry.file_type()?;
        if kind.is_dir() {
            visit(root, &path, out)?;
            continue;
        }
        if !kind.is_file() {
            bail!("build has nonregular artifact: {}", path.display());
        }
        let rel = path
            .strip_prefix(root)?
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("non-UTF8 asset path"))?;
        if rel.contains('\\') || rel.bytes().any(|b| b == 0) {
            bail!("invalid artifact path");
        }
        out.push((rel.replace(std::path::MAIN_SEPARATOR, "/"), path));
        if out.len() > 100_000 {
            bail!("too many build assets");
        }
    }
    Ok(())
}
