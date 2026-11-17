use super::{Arguments, client, http_app};
use anyhow::Result;
pub fn run(args: &Arguments, client: &client::SkryniaClient) -> Result<()> {
    let value = if matches!(args.action.as_str(), "list" | "ls") {
        serde_json::to_value(http_app::apps(client)?)?
    } else {
        let name = args.required("name")?;
        match args.action.as_str() {
            "current" | "inspect" => serde_json::to_value(http_app::current(client, name)?)?,
            "show" => {
                serde_json::to_value(http_app::release(client, name, args.required("release")?)?)?
            }
            "compare" | "diff" => {
                http_app::compare(client, name, args.required("from")?, args.required("to")?)?
            }
            "verify-all" => {
                for entry in http_app::versions(client, name)? {
                    http_app::verify(client, name, &entry.id)?;
                }
                serde_json::json!({"verified":true})
            }
            _ => serde_json::to_value(http_app::versions(client, name)?)?,
        }
    };
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}
