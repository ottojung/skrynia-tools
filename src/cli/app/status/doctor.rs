use super::{Arguments, deploy, format};
use anyhow::{Result, bail};

pub fn run(args: &Arguments) -> Result<()> {
    let root = format::apps_dir(args)?;
    let report = deploy::doctor(&root)?;
    if format::json(args, false)? {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!(
            "Deployment host prerequisites: {}",
            if report.prerequisites_ok {
                "PASS"
            } else {
                "INCOMPLETE"
            }
        );
        println!(
            "Root: {} ({})",
            report.root,
            if report.root_exists {
                "directory"
            } else {
                "missing"
            }
        );
        println!(
            "Git: {}",
            if report.git_available {
                "available"
            } else {
                "unavailable"
            }
        );
        println!(
            "Rootless Podman: {}",
            if report.podman_rootless {
                "available"
            } else {
                "unavailable"
            }
        );
        if !report.missing_directories.is_empty() {
            println!("Not initialized: {}", report.missing_directories.join(", "));
        }
    }
    if !report.prerequisites_ok {
        bail!("deployment prerequisites incomplete");
    }
    Ok(())
}
