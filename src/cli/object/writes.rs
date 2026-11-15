use super::{Arguments, SkryniaClient, Store};
use anyhow::Result;
use std::fs;

pub fn create(client: &SkryniaClient, key: &str, args: &Arguments) -> Result<()> {
    let data = fs::read(args.required("file")?)?;
    println!("{:?}", client.create(key, &data)?);
    Ok(())
}
pub fn replace(client: &SkryniaClient, key: &str, args: &Arguments) -> Result<()> {
    client.replace(
        key,
        &fs::read(args.required("file")?)?,
        args.optional("if-sha256"),
    )
}
pub fn delete(client: &SkryniaClient, key: &str, args: &Arguments) -> Result<()> {
    client.delete(key, args.optional("if-sha256"))
}
