use super::{Arguments, deploy, format};
use anyhow::Result;
use std::path::Path;

pub fn run(args: &Arguments) -> Result<()> {
    let root = Path::new(args.required("apps-dir")?);
    let name = args.required("name")?;
    let versions = deploy::releases(root, name)?;
    if format::json(args, args.action == "releases")? {
        println!("{}", serde_json::to_string_pretty(&versions)?);
        return Ok(());
    }
    let active = deploy::inspect(root, name)?.map(|r| r.id);
    println!("{} — {} available versions", name, versions.len());
    format::heading();
    for entry in &versions {
        format::version(entry, active.as_ref() == Some(&entry.id));
    }
    Ok(())
}
