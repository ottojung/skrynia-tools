use super::{Arguments, changes::Changes, changes_print, deploy, format, tree};
use anyhow::{Result, bail};

pub fn run(args: &Arguments) -> Result<()> {
    let root = format::apps_dir(args)?;
    let name = args.required("name")?;
    let from = args.required("from")?;
    let to = args.required("to")?;
    let first = deploy::release(&root, name, from)?
        .ok_or_else(|| anyhow::anyhow!("source release unavailable"))?;
    let second = deploy::release(&root, name, to)?
        .ok_or_else(|| anyhow::anyhow!("target release unavailable"))?;
    let source = deploy::verify(&root, name, from)?;
    let target = deploy::verify(&root, name, to)?;
    if source.verified == Some(false) || target.verified == Some(false) {
        bail!("release integrity mismatch; refuse to report trusted artifact diff");
    }
    let before = tree::scan(&root.join("releases").join(name).join(from))?;
    let after = tree::scan(&root.join("releases").join(name).join(to))?;
    let changes = Changes::between(&before, &after);
    if format::json(args, false)? {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "app": name, "from": first, "to": second,
                "from_verified": source.verified, "to_verified": target.verified,
                "changes": changes,
            }))?
        );
    } else {
        println!("Artifact changes for {name}: {from} -> {to}");
        changes_print::print(&changes);
    }
    Ok(())
}
