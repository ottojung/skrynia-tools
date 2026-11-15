//! Immutable, local static-host publication of GitHub repository builds.
mod build;
mod files;
mod inspect;
mod process;
mod publish;
mod source;
mod types;
mod validate;
use anyhow::Result;
pub use inspect::{App, apps, inspect, release, releases, rollback};
pub use types::{DeploySpec, Release};
pub use validate::valid_name;

pub fn deploy(spec: &DeploySpec) -> Result<Release> {
    spec.validate()?;
    files::ensure_dirs(&spec.apps_dir)?;
    let _build_lock = process::build_lock(&spec.apps_dir, &spec.name)?;
    process::stale(&spec.apps_dir, &spec.name)?;
    let work =
        spec.apps_dir
            .join(".work")
            .join(format!("{}-{:032x}", spec.name, rand::random::<u128>()));
    std::fs::create_dir_all(&work)?;
    let result = (|| {
        let commit = source::checkout(spec, &work)?;
        build::build(spec, &work)?;
        let entry = publish::prepare(spec, &work, commit)?;
        let _activation_lock = process::lock(&spec.apps_dir, &spec.name)?;
        files::switch(&spec.apps_dir, &spec.name, &entry.id)?;
        Ok(entry)
    })();
    if let Err(error) = std::fs::remove_dir_all(&work) {
        eprintln!("warning: retained temporary build workspace: {error}");
    }
    result
}

#[cfg(test)]
mod tests;
