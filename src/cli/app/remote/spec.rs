use super::{Arguments, publish, rpc};
use anyhow::Result;

pub fn encode(args: &Arguments) -> Result<Vec<u8>> {
    if args.action == "deploy" {
        Ok(serde_json::to_vec(&publish::spec(args)?)?)
    } else {
        Ok(serde_json::to_vec(&rpc::Request::from_args(args)?)?)
    }
}
