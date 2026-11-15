use super::client::{SkryniaClient, Store};
use super::{args::Arguments, secrets};
use anyhow::{Result, bail};
use std::fs;

pub fn run(args: &Arguments) -> Result<()> {
    let mut client = SkryniaClient::new(args.required("url")?)?;
    if let Some(path) = args.optional("token-file") {
        client = client.with_token(secrets::read(path)?);
    }
    let key = args.required("key")?;
    match args.action.as_str() {
        "get" => {
            let bytes = client
                .read(key)?
                .ok_or_else(|| anyhow::anyhow!("object missing"))?;
            if let Some(path) = args.optional("output") {
                fs::write(path, bytes)?;
            } else {
                use std::io::Write;
                std::io::stdout().write_all(&bytes)?;
            }
        }
        "create" => println!(
            "{:?}",
            client.create(key, &fs::read(args.required("file")?)?)?
        ),
        "replace" => client.replace(
            key,
            &fs::read(args.required("file")?)?,
            args.optional("if-sha256"),
        )?,
        "delete" => client.delete(key, args.optional("if-sha256"))?,
        "list" => {
            let mut after: Option<String> = None;
            loop {
                let page = client.list_page(key, after.as_deref())?;
                println!("{}", serde_json::to_string(&page)?);
                after = page.get("next").and_then(|x| x.as_str()).map(str::to_owned);
                if after.is_none() {
                    break;
                }
            }
        }
        _ => bail!("unknown object action"),
    }
    Ok(())
}
