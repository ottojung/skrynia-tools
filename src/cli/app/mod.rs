use super::args::Arguments;
use super::deploy::{self, DeploySpec};
use anyhow::{Result, bail};
use std::path::PathBuf;
mod stdin;

pub fn run(args: &Arguments) -> Result<()> {
    if args.action == "new" {
        return super::scaffold::create(args);
    }
    if args.action == "deploy" && args.optional("spec-stdin") == Some("true") {
        return stdin::deploy();
    }
    let root = PathBuf::from(args.required("apps-dir")?);
    let name = args.required("name")?;
    match args.action.as_str() {
        "deploy" => {
            let spec = DeploySpec {
                name: name.into(),
                repo: args.required("repo")?.into(),
                commit: args.optional("commit").map(str::to_owned),
                subdir: args.optional("subdir").unwrap_or(".").into(),
                apps_dir: root,
                builder_image: args
                    .optional("builder-image")
                    .unwrap_or("docker.io/library/rust:1.93-bookworm")
                    .into(),
            };
            println!("{}", serde_json::to_string_pretty(&deploy::deploy(&spec)?)?);
        }
        "releases" => println!(
            "{}",
            serde_json::to_string_pretty(&deploy::releases(&root, name)?)?
        ),
        "inspect" => println!(
            "{}",
            serde_json::to_string_pretty(&deploy::inspect(&root, name)?)?
        ),
        "rollback" => {
            deploy::rollback(&root, name, args.required("release")?)?;
            println!("rollback activated");
        }
        _ => bail!("unknown app action"),
    }
    Ok(())
}
