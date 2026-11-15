use super::{Arguments, compare, history, inventory, status};
use anyhow::{Result, bail};

pub fn run(args: &Arguments) -> Result<()> {
    match args.action.as_str() {
        "list" | "ls" => inventory::run(args),
        "versions" | "history" | "releases" => history::run(args),
        "current" | "inspect" | "show" | "verify" | "rollback" | "activate" => status::run(args),
        "compare" => compare::run(args),
        "doctor" => status::doctor::run(args),
        _ => bail!("unsupported remote app action"),
    }
}
