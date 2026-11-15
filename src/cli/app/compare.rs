use super::{Arguments, deploy, format};
use anyhow::Result;
use std::path::Path;

pub fn run(args: &Arguments) -> Result<()> {
    let root = Path::new(args.required("apps-dir")?);
    let name = args.required("name")?;
    let before = deploy::release(root, name, args.required("from")?)?
        .ok_or_else(|| anyhow::anyhow!("source release is not available"))?;
    let after = deploy::release(root, name, args.required("to")?)?
        .ok_or_else(|| anyhow::anyhow!("target release is not available"))?;
    if format::json(args, false)? {
        println!(
            "{}",
            serde_json::json!({"metadata_only": true, "from": before, "to": after})
        );
        return Ok(());
    }
    println!("Release provenance comparison (not a file-content diff)");
    println!("{:<14} {:<40} TO", "FIELD", "FROM");
    println!("{:<14} {:<40} {}", "Release", before.id, after.id);
    println!("{:<14} {:<40} {}", "Commit", before.commit, after.commit);
    println!("{:<14} {:<40} {}", "Repository", before.repo, after.repo);
    println!(
        "{:<14} {:<40} {}",
        "Subdirectory", before.subdir, after.subdir
    );
    println!(
        "{:<14} {:<40} {}",
        "Builder image", before.builder_image, after.builder_image
    );
    println!(
        "{:<14} {:<40} {}",
        "Published",
        format::date(&before),
        format::date(&after)
    );
    Ok(())
}
