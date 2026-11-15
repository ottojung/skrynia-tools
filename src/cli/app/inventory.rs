use super::{Arguments, deploy, format};
use anyhow::Result;
use std::path::Path;

pub fn run(args: &Arguments) -> Result<()> {
    let root = Path::new(args.required("apps-dir")?);
    let apps = deploy::apps(root)?;
    if format::json(args, false)? {
        println!("{}", serde_json::to_string_pretty(&apps)?);
        return Ok(());
    }
    println!(
        "{:<25} {:<29} {:<12} SOURCE",
        "APP", "ACTIVE RELEASE", "COMMIT"
    );
    for app in &apps {
        if let Some(entry) = &app.active {
            println!(
                "{:<25} {:<29} {:<12} {} [{}]",
                app.name,
                entry.id,
                &entry.commit[..12],
                entry.repo,
                entry.subdir
            );
        } else {
            println!("{:<25} (no active release)", app.name);
        }
    }
    Ok(())
}
