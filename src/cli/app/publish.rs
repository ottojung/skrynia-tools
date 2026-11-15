use super::{Arguments, deploy};
use anyhow::Result;
use std::path::PathBuf;

pub fn run(args: &Arguments) -> Result<()> {
    let spec = deploy::DeploySpec {
        name: args.required("name")?.into(),
        repo: args.required("repo")?.into(),
        commit: args.optional("commit").map(str::to_owned),
        subdir: args.optional("subdir").unwrap_or(".").into(),
        apps_dir: PathBuf::from(args.required("apps-dir")?),
        builder_image: args
            .optional("builder-image")
            .unwrap_or("docker.io/library/rust:1.93-bookworm")
            .into(),
    };
    println!("{}", serde_json::to_string_pretty(&deploy::deploy(&spec)?)?);
    Ok(())
}
