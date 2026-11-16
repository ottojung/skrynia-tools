use super::{Item, Report, deploy, files};
use anyhow::Result;
use std::path::Path;

pub fn scan(root: &Path, name: &str) -> Result<Report> {
    let names = files::names(root, name)?;
    let mut report = Report {
        app: name.into(),
        ..Report::default()
    };
    for filename in names {
        let id = filename.strip_suffix(".json").unwrap_or("");
        if !deploy::valid_release_id(id) {
            report.errors += 1;
            report.releases.push(Item {
                release: filename,
                verification: None,
                error: Some("invalid release metadata filename".into()),
            });
            continue;
        }
        let item = match deploy::verify(root, name, id) {
            Ok(verification) => {
                match verification.verified {
                    Some(true) => report.verified += 1,
                    Some(false) => report.mismatched += 1,
                    None => report.unverifiable += 1,
                };
                Item {
                    release: id.to_owned(),
                    verification: Some(verification),
                    error: None,
                }
            }
            Err(error) => {
                report.errors += 1;
                Item {
                    release: id.to_owned(),
                    verification: None,
                    error: Some(error.to_string()),
                }
            }
        };
        report.releases.push(item);
    }
    Ok(report)
}
