use super::{args::Arguments, deploy};
use anyhow::{Result, bail};
mod compare;
mod format;
mod history;
mod inventory;
mod publish;
mod remote;
mod rpc;
mod status;
mod stdin;

pub fn run(args: &Arguments) -> Result<()> {
    if args.action == "rpc" {
        return rpc::serve(args);
    }
    if remote::requested(args)? {
        return remote::run(args);
    }
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
        "diff" => compare::diff(args),
        "doctor" => status::doctor::run(args),
        "audit" => status::audit::run(args),
        "versions" | "history" | "releases" => history::run(args),
        "current" | "inspect" | "show" | "verify" | "verify-all" | "rollback" | "activate" => {
            status::run(args)
        }
        _ => bail!("unknown app action"),
    }
}
