use super::client;
use super::{Asset, Keys, Release, activate, create_exact, scan};
use anyhow::Result;
use std::path::Path;
pub fn deploy(
    client: &client::SkryniaClient,
    name: &str,
    dir: &Path,
    repo: &str,
    commit: &str,
    subdir: &str,
) -> Result<Release> {
    let keys = Keys::new(name)?;
    let id = format!("{:032x}", rand::random::<u128>());
    let mut files = Vec::new();
    for (path, file) in scan(dir)? {
        let bytes = std::fs::read(file)?;
        let key = keys.asset(&id, &path)?;
        create_exact(client, &key, &bytes)?;
        files.push(Asset {
            path,
            key,
            sha256: client::SkryniaClient::digest(&bytes),
            bytes: bytes.len() as u64,
        });
    }
    let entry = Release {
        schema: 1,
        name: name.into(),
        id: id.clone(),
        commit: commit.into(),
        repo: repo.into(),
        subdir: subdir.into(),
        files,
    };
    create_exact(client, &keys.version(&id)?, &serde_json::to_vec(&entry)?)?;
    create_exact(client, &keys.entry(), super::assets::landing().as_bytes())?;
    create_exact(client, &keys.stamp(&id)?, b"ready")?;
    activate(client, name, &id)?;
    Ok(entry)
}
