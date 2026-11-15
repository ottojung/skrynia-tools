use super::{Arguments, SkryniaClient};
use anyhow::Result;
use std::fs;
use std::io::Write;

pub fn get(client: &SkryniaClient, key: &str, args: &Arguments) -> Result<()> {
    let bytes = client
        .read(key)?
        .ok_or_else(|| anyhow::anyhow!("object missing"))?;
    if let Some(path) = args.optional("output") {
        fs::write(path, bytes)?;
    } else {
        std::io::stdout().write_all(&bytes)?;
    }
    Ok(())
}
pub fn list(client: &SkryniaClient, matcher: &str) -> Result<()> {
    let mut after: Option<String> = None;
    loop {
        let page = client.list_page(matcher, after.as_deref())?;
        println!("{}", serde_json::to_string(&page)?);
        let next = page.get("next").and_then(|x| x.as_str()).map(str::to_owned);
        if next.as_deref() <= after.as_deref() && next.is_some() {
            anyhow::bail!("non-advancing Skrynia listing cursor");
        }
        after = next;
        if after.is_none() {
            break;
        }
    }
    Ok(())
}
