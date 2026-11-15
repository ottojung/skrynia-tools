use super::{
    files,
    types::{DeploySpec, Release},
};
use anyhow::{Result, bail};
use std::{fs, path::Path};

pub fn activate(spec: &DeploySpec, work: &Path, commit: String) -> Result<Release> {
    let input = work.join("source").join(&spec.subdir).join("build");
    if !input.join("index.html").is_file() {
        bail!("build/index.html is required");
    }
    let releases = spec.apps_dir.join("releases").join(&spec.name);
    let metadata = spec.apps_dir.join("metadata").join(&spec.name);
    files::checked_dir(&releases)?;
    files::checked_dir(&metadata)?;
    let id = format!("{}-{:016x}", &commit[..12], rand::random::<u64>());
    let stage = releases.join(format!(".{id}.staging"));
    files::copy_tree(&input, &stage)?;
    let entry = Release {
        id: id.clone(),
        name: spec.name.clone(),
        repo: spec.repo.clone(),
        commit,
        subdir: spec.subdir.clone(),
        builder_image: spec.builder_image.clone(),
    };
    let record = metadata.join(format!("{id}.json"));
    fs::write(&record, serde_json::to_vec_pretty(&entry)?)?;
    files::sync_file(&record)?;
    files::sync_dir(&metadata)?;
    fs::rename(stage, releases.join(&id))?;
    files::sync_dir(&releases)?;
    files::switch(&spec.apps_dir, &spec.name, &id)?;
    Ok(entry)
}
