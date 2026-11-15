use super::{Arguments, deploy, format};
use anyhow::{Result, bail};
use std::path::Path;

pub fn run(args: &Arguments, root: &Path, name: &str) -> Result<()> {
    let id = if let Some(id) = args.optional("release") {
        id.to_owned()
    } else {
        deploy::inspect(root, name)?
            .ok_or_else(|| anyhow::anyhow!("no active release"))?
            .id
    };
    let report = deploy::verify(root, name, &id)?;
    if format::json(args, false)? {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        let status = match report.verified {
            Some(true) => "VERIFIED",
            Some(false) => "MISMATCH",
            None => "UNVERIFIABLE (legacy release)",
        };
        println!("{status}: {name} release {id}");
        println!(
            "Expected: {}\nActual: {}",
            report
                .expected_sha256
                .as_deref()
                .unwrap_or("(not recorded)"),
            report.actual_sha256
        );
    }
    if report.verified != Some(true) {
        bail!("release integrity not confirmed");
    }
    Ok(())
}
