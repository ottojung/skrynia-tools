use super::args::Arguments;
use anyhow::{Result, bail};

pub fn run(args: &Arguments) -> Result<()> {
    if args.action == "inspect" {
        return super::provision::inspect(args);
    }
    if args.action == "resume" {
        return super::provision::resume(args);
    }
    if !matches!(args.action.as_str(), "plan" | "create") {
        bail!("unknown admin action");
    }
    let name = args.required("name")?;
    if !super::deploy::valid_name(name) {
        bail!("invalid app name");
    }
    let bytes = args.number("max-bytes")?;
    let objects = args.number("max-objects")?;
    let id = args
        .optional("id")
        .map(str::to_owned)
        .unwrap_or_else(|| format!("{:032x}", rand::random::<u128>()));
    if id.len() != 32 || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
        bail!("--id must be a 32-character hex identifier");
    }
    let prefix = format!("a.{name}.{id}.");
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
