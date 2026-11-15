use super::{Arguments, deploy, dispatch, guard, request::Request};
use anyhow::{Result, bail};
use std::io::Read;

pub fn serve(args: &Arguments) -> Result<()> {
    if args.optional("spec-stdin") != Some("true") {
        bail!("app rpc requires --spec-stdin true");
    }
    if std::env::var_os("SKRYNIA_DEPLOY_ROOT").is_none() {
        bail!("app rpc requires SKRYNIA_DEPLOY_ROOT authorization");
    }
    let mut bytes = Vec::new();
    std::io::stdin().take(16385).read_to_end(&mut bytes)?;
    if bytes.len() > 16384 {
        bail!("remote request exceeds 16 KiB");
    }
    let value: serde_json::Value = serde_json::from_slice(&bytes)?;
    if value.get("action").is_none() {
        let spec: deploy::DeploySpec = serde_json::from_value(value)?;
        guard::enforce_root(&spec.apps_dir)?;
        println!("{}", serde_json::to_string_pretty(&deploy::deploy(&spec)?)?);
    } else {
        let request: Request = serde_json::from_value(value)?;
        guard::enforce_root(&request.apps_dir)?;
        dispatch::run(&request.into_args())?;
    }
    Ok(())
}
