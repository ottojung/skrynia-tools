use super::{args::Arguments, deploy};
use anyhow::{Result, bail};
mod compare;
mod format;
mod guard;
mod history;
mod inventory;
mod publish;
mod status;
mod stdin;

pub fn run(args: &Arguments) -> Result<()> {
    match args.action.as_str() {
        "help" | "--help" => {
            super::help::print();
            Ok(())
        }
        "new" => super::scaffold::create(args),
        "deploy" if args.optional("spec-stdin") == Some("true") => stdin::deploy(),
        "deploy" => publish::run(args),
        "list" | "ls" => inventory::run(args),
        "compare" => compare::run(args),
        "versions" | "history" | "releases" => history::run(args),
        "current" | "inspect" | "show" | "rollback" | "activate" => status::run(args),
        _ => bail!("unknown app action"),
    }
}
