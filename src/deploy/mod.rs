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
pub use inspect::{inspect, releases, rollback};
pub use types::{DeploySpec, Release};
pub use validate::valid_name;

pub fn deploy(spec: &DeploySpec) -> Result<Release> {
    spec.validate()?;
    validate::ensure_dirs(&spec.apps_dir)?;
    let _lock = process::lock(&spec.apps_dir, &spec.name)?;
    let work =
        spec.apps_dir
            .join(".work")
            .join(format!("{}-{:032x}", spec.name, rand::random::<u128>()));
    std::fs::create_dir_all(&work)?;
    let result = (|| {
        let commit = source::checkout(spec, &work)?;
        build::build(spec, &work)?;
        publish::activate(spec, &work, commit)
    })();
    let cleanup = std::fs::remove_dir_all(&work);
    if result.is_ok() {
        cleanup?;
    }
    result
}

#[cfg(test)]
mod tests;
