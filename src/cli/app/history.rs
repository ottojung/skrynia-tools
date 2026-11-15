use super::{Arguments, deploy, format};
use anyhow::Result;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct Version<'a> {
    #[serde(flatten)]
    release: &'a deploy::Release,
    active: bool,
}
pub fn run(args: &Arguments) -> Result<()> {
    let root = Path::new(args.required("apps-dir")?);
    let name = args.required("name")?;
    let versions = deploy::releases(root, name)?;
    let active = deploy::inspect(root, name)?.map(|r| r.id);
    if format::json(args, args.action == "releases")? {
        if args.action == "releases" {
            println!("{}", serde_json::to_string_pretty(&versions)?);
        } else {
            let rows: Vec<_> = versions
                .iter()
                .map(|release| Version {
                    release,
                    active: active.as_ref() == Some(&release.id),
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&rows)?);
        }
        return Ok(());
    }
    println!("{} — {} available versions", name, versions.len());
    format::heading();
    for entry in &versions {
        format::version(entry, active.as_ref() == Some(&entry.id));
    }
    Ok(())
}
