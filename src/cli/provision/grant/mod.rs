mod auth;
mod create;
mod options;
mod path;
mod reuse;
use super::{Arguments, SkryniaClient, secrets};
use anyhow::Result;
pub fn run(args: &Arguments) -> Result<()> {
    let url = args.required("url")?;
    let client = SkryniaClient::new(url)?;
    let existing = args.optional("existing-grant");
    let (plan, token) = if existing.is_none() {
        let plan = options::parse(args)?;
        let token = auth::parent(args, &client)?;
        (Some(plan), Some(token))
    } else {
        (None, None)
    };
    let path = path::destination(args)?;
    let _lock = secrets::lock(&path)?;
    if let Some(file) = existing {
        return reuse::issue(args, &client, &path, file);
    }
    let (locations, operations, bytes, objects) = plan.unwrap();
    let token = token.unwrap();
    let client = client.with_token(token);
    create::run(&client, &path, &locations, &operations, bytes, objects)
}
