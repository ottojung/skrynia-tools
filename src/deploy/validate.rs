use anyhow::{Result, bail};
use std::path::{Component, Path};

pub fn valid_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}
pub fn valid_rel(path: &str) -> bool {
    !path.is_empty()
        && Path::new(path)
            .components()
            .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
}
pub fn validate_repo(repo: &str) -> Result<()> {
    let path = repo
        .strip_prefix("https://github.com/")
        .or_else(|| repo.strip_prefix("git@github.com:"))
        .ok_or_else(|| anyhow::anyhow!("expected GitHub HTTPS or scp-style SSH repository"))?;
    let path = path.strip_suffix(".git").unwrap_or(path);
    let parts: Vec<_> = path.split('/').collect();
    if parts.len() != 2
        || parts.iter().any(|p| {
            p.is_empty()
                || !p
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
        })
    {
        bail!("repository must be github.com/owner/repository");
    }
    Ok(())
}

pub fn valid_release_id(id: &str) -> bool {
    id.len() == 29
        && id.as_bytes()[12] == b'-'
        && id
            .bytes()
            .enumerate()
            .all(|(i, b)| i == 12 || b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
