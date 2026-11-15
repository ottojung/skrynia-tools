use super::{Arguments, deploy, format};
use anyhow::Result;

pub fn spec(args: &Arguments) -> Result<deploy::DeploySpec> {
    Ok(deploy::DeploySpec {
        name: args.required("name")?.into(),
        repo: args.required("repo")?.into(),
        commit: args.optional("commit").map(str::to_owned),
        subdir: args.optional("subdir").unwrap_or(".").into(),
        apps_dir: format::apps_dir(args)?,
        builder_image: args
            .optional("builder-image")
            .unwrap_or("docker.io/library/rust:1.93-bookworm")
            .into(),
    })
}
pub fn run(args: &Arguments) -> Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(&deploy::deploy(&spec(args)?)?)?
    );
    Ok(())
}
