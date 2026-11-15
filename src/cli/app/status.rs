use super::{Arguments, deploy, format};
use anyhow::{Result, bail};

pub fn run(args: &Arguments) -> Result<()> {
    let root = format::apps_dir(args)?;
    let name = args.required("name")?;
    if matches!(args.action.as_str(), "rollback" | "activate") {
        let id = args.required("release")?;
        deploy::rollback(&root, name, id)?;
        if format::json(args, false)? {
            println!("{}", serde_json::json!({"app":name,"activated":id}));
        } else {
            println!("Activated {name} release {id}");
        }
        return Ok(());
    }
    let entry = match args.action.as_str() {
        "show" => Some(
            deploy::release(&root, name, args.required("release")?)?
                .ok_or_else(|| anyhow::anyhow!("release is not available"))?,
        ),
        "current" | "inspect" => deploy::inspect(&root, name)?,
        _ => bail!("unknown app status action"),
    };
    if format::json(args, args.action == "inspect" || args.action == "show")? {
        println!("{}", serde_json::to_string_pretty(&entry)?);
    } else if let Some(entry) = &entry {
        format::heading();
        format::version(entry, args.action != "show");
    } else {
        println!("No active release");
    }
    Ok(())
}
