use super::{Arguments, Result, Value, secrets};
use std::path::Path;
mod summary;
pub use summary::summary;
pub fn inspect(args: &Arguments) -> Result<()> {
    let path = Path::new(args.required("credentials-file")?);
    let data: Value = serde_json::from_str(&secrets::read(
        path.to_str()
            .ok_or_else(|| anyhow::anyhow!("invalid credential path"))?,
    )?)?;
    println!("{}", serde_json::to_string_pretty(&summary(&data)?)?);
    Ok(())
}
