use super::{args::Arguments, secrets};
use super::{client::SkryniaClient, protocol_log::AppendLog};
use anyhow::{Result, bail};

pub fn run(args: &Arguments) -> Result<()> {
    let mut client = SkryniaClient::new(args.required("url")?)?;
    if let Some(path) = args.optional("token-file") {
        client = client.with_token(secrets::read(path)?);
    }
    let log = AppendLog::open(&client, args.required("prefix")?)?;
    match args.action.as_str() {
        "init" => log.initialize()?,
        "append" => {
            let id = args.required("operation-id")?;
            let data = std::fs::read(args.required("file")?)?;
            println!("{}", log.append(id, &data)?);
        }
        "read" => {
            let start = args.optional("from").unwrap_or("1").parse()?;
            for (position, id, payload) in log.read_from(start, 1000)? {
                println!(
                    "{}",
                    serde_json::json!({"position":position,"operation_id":id,
                    "payload_hex":hex::encode(payload)})
                );
            }
        }
        _ => bail!("unknown log action"),
    }
    Ok(())
}
