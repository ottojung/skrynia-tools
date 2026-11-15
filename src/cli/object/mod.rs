mod reads;
mod writes;
use super::client::{SkryniaClient, Store};
use super::{args::Arguments, secrets};
use anyhow::{Result, bail};

pub fn run(args: &Arguments) -> Result<()> {
    let mut client = SkryniaClient::new(args.required("url")?)?;
    if let Some(path) = args.optional("token-file") {
        client = client.with_token(secrets::read(path)?);
    }
    let key = if args.action == "list" {
        args.locations()?
    } else {
        args.required("key")?
    };
    match args.action.as_str() {
        "get" => reads::get(&client, key, args),
        "list" => reads::list(&client, key),
        "create" => writes::create(&client, key, args),
        "replace" => writes::replace(&client, key, args),
        "delete" => writes::delete(&client, key, args),
        _ => bail!("unknown object action"),
    }
}
