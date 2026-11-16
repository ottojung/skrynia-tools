use super::{Arguments, format};
use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub action: String,
    pub apps_dir: PathBuf,
    pub name: Option<String>,
    pub release: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub format: Option<String>,
}
impl Request {
    pub fn from_args(args: &Arguments) -> Result<Self> {
        let action = args.action.clone();
        if !matches!(
            action.as_str(),
            "list"
                | "ls"
                | "versions"
                | "history"
                | "releases"
                | "current"
                | "inspect"
                | "show"
                | "rollback"
                | "activate"
                | "compare"
                | "verify"
                | "doctor"
                | "audit"
        ) {
            bail!("unsupported remote app action");
        }
        Ok(Self {
            action,
            apps_dir: format::apps_dir(args)?,
            name: args.optional("name").map(str::to_owned),
            release: args.optional("release").map(str::to_owned),
            from: args.optional("from").map(str::to_owned),
            to: args.optional("to").map(str::to_owned),
            format: args.optional("format").map(str::to_owned),
        })
    }
}
