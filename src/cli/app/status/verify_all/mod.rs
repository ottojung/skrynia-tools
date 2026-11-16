use super::{Arguments, deploy, format};
use anyhow::{Result, bail};
use serde::Serialize;
use std::path::Path;
mod scan;

#[derive(Debug, Serialize)]
pub struct Item {
    pub release: String,
    pub verification: Option<deploy::Verification>,
    pub error: Option<String>,
}
#[derive(Debug, Default, Serialize)]
pub struct Report {
    pub app: String,
    pub verified: usize,
    pub mismatched: usize,
    pub unverifiable: usize,
    pub errors: usize,
    pub releases: Vec<Item>,
}
pub fn run(args: &Arguments, root: &Path, name: &str) -> Result<()> {
    let report = scan::scan(root, name)?;
    if format::json(args, false)? {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!(
            "Integrity of {}: {} verified, {} mismatched, {} unverifiable, {} errors",
            name, report.verified, report.mismatched, report.unverifiable, report.errors
        );
        for item in &report.releases {
            let status = match (&item.verification, &item.error) {
                (_, Some(error)) => format!("ERROR: {error}"),
                (Some(verification), _) => match verification.verified {
                    Some(true) => "VERIFIED".into(),
                    Some(false) => "MISMATCH".into(),
                    None => "UNVERIFIABLE".into(),
                },
                _ => "ERROR: no result".into(),
            };
            println!("{}: {status}", item.release);
        }
    }
    if report.verified != report.releases.len() {
        bail!("one or more releases could not be verified");
    }
    Ok(())
}
