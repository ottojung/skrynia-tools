use anyhow::{Result, bail};
pub fn github(repo: &str) -> Result<()> {
    let Some(slug) = repo.strip_prefix("https://github.com/") else {
        bail!("--repo must be a GitHub HTTPS repository URL");
    };
    let slug = slug.strip_suffix(".git").unwrap_or(slug);
    if slug.split('/').count() != 2
        || slug.split('/').any(|part| {
            part.is_empty()
                || !part
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
        })
    {
        bail!("--repo must be a GitHub HTTPS repository URL");
    }
    Ok(())
}
