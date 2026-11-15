use super::{DeploySpec, Release, files, record};
use anyhow::{Result, bail};
use std::{fs, path::Path};

pub fn prepare(spec: &DeploySpec, work: &Path, commit: String) -> Result<Release> {
    let input = work.join("source").join(&spec.subdir).join("build");
    let releases = spec.apps_dir.join("releases").join(&spec.name);
    let metadata = spec.apps_dir.join("metadata").join(&spec.name);
    files::checked_dir(&releases)?;
    files::checked_dir(&metadata)?;
    let id = format!("{}-{:016x}", &commit[..12], rand::random::<u64>());
    let stage = releases.join(format!(".{id}.staging"));
    files::copy_tree(&input, &stage)?;
    let content_sha256 = Some(files::digest_tree(&stage)?);
    let entry = Release {
        id: id.clone(),
        name: spec.name.clone(),
        repo: spec.repo.clone(),
        commit,
        subdir: spec.subdir.clone(),
        builder_image: spec.builder_image.clone(),
        content_sha256,
        published_unix_ms: Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis()
                .try_into()?,
        ),
    };
    if fs::symlink_metadata(releases.join(&id)).is_ok() {
        bail!("release identifier already exists");
    }
    record::save(&metadata, &entry)?;
    fs::rename(stage, releases.join(&id))?;
    files::sync_dir(&releases)?;
    Ok(entry)
}
