use super::{Arguments, deploy::Release};
use anyhow::{Result, bail};
use chrono::{DateTime, Utc};

pub fn json(args: &Arguments, default: bool) -> Result<bool> {
    match args.optional("format") {
        None => Ok(default),
        Some("json") => Ok(true),
        Some("table") => Ok(false),
        _ => bail!("--format must be json or table"),
    }
}
pub fn heading() {
    println!(
        "  {:<29} {:<12} {:<20} SOURCE",
        "RELEASE", "COMMIT", "DEPLOYED (UTC)"
    );
}
pub fn date(entry: &Release) -> String {
    entry
        .published_unix_ms
        .and_then(|n| i64::try_from(n).ok())
        .and_then(DateTime::<Utc>::from_timestamp_millis)
        .map(|v| v.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "unknown".into())
}
pub fn version(entry: &Release, active: bool) {
    let mark = if active { "*" } else { " " };
    println!(
        "{mark} {:<29} {:<12} {:<20} {} [{}]",
        entry.id,
        &entry.commit[..12],
        date(entry),
        entry.repo,
        entry.subdir
    );
}
