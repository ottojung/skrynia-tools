use super::args::Arguments;
use anyhow::{Result, bail};

pub fn run(args: &Arguments) -> Result<()> {
    if !matches!(args.action.as_str(), "plan" | "create") {
        bail!("unknown admin action");
    }
    let name = args.required("name")?;
    if !super::deploy::valid_name(name) {
        bail!("invalid app name");
    }
    let bytes = args.number("max-bytes")?;
    let objects = args.number("max-objects")?;
    let prefix = format!("a.{name}.{:032x}.", rand::random::<u128>());
    let matcher = format!("{prefix}*");
    let ops = [
        "create",
        "modify",
        "delete",
        "list",
        "delegate",
        "subscribe",
    ];
    let plan = serde_json::json!({"name": name,"prefix":prefix,"locations":matcher,
        "operations":ops, "maxBytes":bytes, "maxObjects":objects});
    if args.action == "plan" {
        println!("{}", serde_json::to_string_pretty(&plan)?);
        return Ok(());
    }
    super::provision::create(args, name, &prefix, &matcher, &ops, bytes, objects)
}
