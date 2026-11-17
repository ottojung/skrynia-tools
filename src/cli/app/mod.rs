mod inspect;
mod source;
mod stable;
use super::{Arguments, client, http_app};
use anyhow::{Result, bail};
fn client(args: &Arguments) -> Result<client::SkryniaClient> {
    let url = args.required("url")?;
    let variable = args.optional("token-env").unwrap_or("SKRYNIA_TOKEN");
    let token = std::env::var(variable)
        .map_err(|_| anyhow::anyhow!("missing deployment grant in {variable}"))?;
    if token.is_empty() {
        bail!("empty deployment token");
    }
    Ok(client::SkryniaClient::new(url)?.with_token(token))
}
pub fn run(args: &Arguments) -> Result<()> {
    if args.action == "new" {
        return super::scaffold::create(args);
    }
    let client = client(args)?;
    match args.action.as_str() {
        "deploy" => {
            let name = args.required("name")?;
            let built = source::prepare(args)?;
            let release = http_app::deploy(
                &client,
                name,
                &built.directory,
                args.optional("repo").unwrap_or(""),
                args.optional("commit").unwrap_or(""),
                args.optional("subdir").unwrap_or("."),
            )?;
            println!("{}", serde_json::to_string_pretty(&release)?);
            Ok(())
        }
        "rollback" | "activate" => {
            let pointer =
                http_app::activate(&client, args.required("name")?, args.required("release")?)?;
            println!("{}", serde_json::to_string_pretty(&pointer)?);
            Ok(())
        }
        "publish-stable" => stable::run(args),
        "verify" => http_app::verify(&client, args.required("name")?, args.required("release")?),
        "current" | "inspect" | "show" | "versions" | "history" | "releases" | "list" | "ls"
        | "compare" | "diff" | "verify-all" => inspect::run(args, &client),
        _ => bail!("unknown app command: {}", args.action),
    }
}
