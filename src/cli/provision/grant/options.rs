use super::Arguments;
use anyhow::{Result, bail};
pub fn parse(args: &Arguments) -> Result<(String, Vec<String>, u64, u64)> {
    let locations = args.required("locations")?.to_owned();
    if locations.is_empty() {
        bail!("grant locations cannot be empty");
    }
    let bytes = args.number("max-bytes")?;
    let objects = args.number("max-objects")?;
    if !(2048..=9_007_199_254_740_991).contains(&bytes)
        || !(1..=9_007_199_254_740_991).contains(&objects)
    {
        bail!("grant quotas out of Skrynia's supported range");
    }
    let spec = args.optional("operations").unwrap_or("create,modify,list");
    let mut operations = Vec::new();
    for op in spec.split(',') {
        if !matches!(
            op,
            "create" | "modify" | "delete" | "list" | "delegate" | "subscribe"
        ) {
            bail!("invalid grant operation: {op}");
        }
        if operations.iter().any(|x| x == op) {
            bail!("duplicate grant operation");
        }
        operations.push(op.to_owned());
    }
    if operations.is_empty() {
        bail!("grant operations cannot be empty");
    }
    Ok((locations, operations, bytes, objects))
}
