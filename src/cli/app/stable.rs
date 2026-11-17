use super::{Arguments, http_app};
use anyhow::Result;
pub fn run(args: &Arguments) -> Result<()> {
    let client = super::client(args)?;
    let bytes = std::fs::read(args.required("file")?)?;
    let key = http_app::publish_stable(
        &client,
        args.required("name")?,
        args.required("asset")?,
        &bytes,
    )?;
    println!(
        "{}",
        serde_json::json!({
            "key": key, "sha256": crate::client::SkryniaClient::digest(&bytes)
        })
    );
    Ok(())
}
