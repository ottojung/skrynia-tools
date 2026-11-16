use super::{Arguments, deploy, format};
use anyhow::{Result, bail};

pub fn run(args: &Arguments) -> Result<()> {
    let root = format::apps_dir(args)?;
    let report = deploy::audit(&root, args.required("name")?)?;
    if format::json(args, false)? {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        if let Some(error) = &report.active_error {
            println!("Active symlink error: {error}");
        }
        if let Some(error) = &report.generation_error {
            println!("Activation generation error: {error}");
        }
        println!(
            "Audit {}: {} retained releases, {}",
            report.name,
            report.available_releases,
            if report.clean { "clean" } else { "incomplete" }
        );
        for (kind, items) in [
            ("Missing artifact", &report.metadata_without_artifact),
            ("Missing metadata", &report.artifact_without_metadata),
            ("Abandoned staging", &report.abandoned_staging),
            (
                "Abandoned activation link",
                &report.abandoned_activation_links,
            ),
            ("Abandoned build workspace", &report.abandoned_workspaces),
            ("Invalid entry", &report.invalid_entries),
        ] {
            for item in items {
                println!("{kind}: {item}");
            }
        }
    }
    if !report.clean {
        bail!("release audit found incomplete records; no files modified");
    }
    Ok(())
}
