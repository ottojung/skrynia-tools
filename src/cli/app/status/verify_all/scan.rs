use super::{Item, Report, deploy};
use anyhow::{Result, bail};
use std::path::Path;

pub fn scan(root: &Path, name: &str) -> Result<Report> {
    let releases = deploy::releases(root, name)?;
    if releases.is_empty() {
        bail!("no retained releases to verify");
    }
    let mut report = Report {
        app: name.into(),
        ..Report::default()
    };
    for release in releases {
        let item = match deploy::verify(root, name, &release.id) {
            Ok(verification) => {
                match verification.verified {
                    Some(true) => report.verified += 1,
                    Some(false) => report.mismatched += 1,
                    None => report.unverifiable += 1,
                };
                Item {
                    release: release.id,
                    verification: Some(verification),
                    error: None,
                }
            }
            Err(error) => {
                report.errors += 1;
                Item {
                    release: release.id,
                    verification: None,
                    error: Some(error.to_string()),
                }
            }
        };
        report.releases.push(item);
    }
    Ok(report)
}
